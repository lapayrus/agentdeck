use std::collections::HashMap;

use crate::model::{Artifact, ArtifactType, Ecosystem, ShadowedArtifact};
use crate::source::Candidate;

fn type_order(artifact_type: ArtifactType) -> u8 {
    match artifact_type {
        ArtifactType::Agent => 0,
        ArtifactType::Skill => 1,
        ArtifactType::Mcp => 2,
        ArtifactType::Hook => 3,
        ArtifactType::Command => 4,
        ArtifactType::Plugin => 5,
        ArtifactType::Memory => 6,
        ArtifactType::Settings => 7,
    }
}

fn scope_label(scope: crate::model::Scope) -> &'static str {
    match scope {
        crate::model::Scope::Global => "global",
        crate::model::Scope::Project => "project",
        crate::model::Scope::ProjectLocal => "project-local",
    }
}

pub fn resolve(candidates: Vec<Candidate>, warnings: &mut Vec<String>) -> Vec<Artifact> {
    let mut groups: HashMap<(Ecosystem, ArtifactType, String), Vec<Candidate>> = HashMap::new();
    for candidate in candidates {
        groups
            .entry((
                candidate.artifact.ecosystem,
                candidate.artifact.artifact_type,
                candidate.artifact.name.to_lowercase(),
            ))
            .or_default()
            .push(candidate);
    }

    let mut winners: Vec<Candidate> = Vec::new();
    for (_, mut group) in groups {
        group.sort_by(|left, right| {
            left.rank
                .cmp(&right.rank)
                .then_with(|| left.artifact.primary_path.cmp(&right.artifact.primary_path))
        });
        let mut winner = group.remove(0);
        for loser in group {
            if loser.rank == winner.rank {
                winner.artifact.warnings.push(format!(
                    "Another definition in the same scope also uses the name \"{}\"; the winner is not deterministic.",
                    winner.artifact.name
                ));
            }
            winner.artifact.shadowed.push(ShadowedArtifact {
                name: loser.artifact.name,
                path: loser.artifact.primary_path,
                scope: loser.artifact.scope,
                ecosystem: loser.artifact.ecosystem,
                reason: format!(
                    "Overridden by the {} definition at {}.",
                    scope_label(winner.artifact.scope),
                    winner.artifact.primary_path
                ),
            });
        }
        winners.push(winner);
    }

    let mut merged: Vec<Candidate> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for winner in winners {
        if let Some(position) = index.get(&winner.link).copied() {
            let existing = &mut merged[position];
            for ecosystem in &winner.artifact.ecosystems {
                if !existing.artifact.ecosystems.contains(ecosystem) {
                    existing.artifact.ecosystems.push(*ecosystem);
                }
            }
            for path in &winner.artifact.source_paths {
                if !existing.artifact.source_paths.contains(path) {
                    existing.artifact.source_paths.push(path.clone());
                }
            }
            for shadowed in winner.artifact.shadowed {
                existing.artifact.shadowed.push(shadowed);
            }
            for warning in winner.artifact.warnings {
                if !existing.artifact.warnings.contains(&warning) {
                    existing.artifact.warnings.push(warning);
                }
            }
            if existing.artifact.disable_mechanism.is_none() {
                existing.artifact.disable_mechanism = winner.artifact.disable_mechanism;
            } else if let Some(mechanism) = winner.artifact.disable_mechanism {
                if !existing
                    .artifact
                    .disable_mechanism
                    .as_deref()
                    .unwrap_or_default()
                    .contains(&mechanism)
                {
                    let current = existing.artifact.disable_mechanism.take().unwrap();
                    existing.artifact.disable_mechanism = Some(format!("{current}; {mechanism}"));
                }
            }
            existing.artifact.can_disable |= winner.artifact.can_disable;
        } else {
            index.insert(winner.link.clone(), merged.len());
            merged.push(winner);
        }
    }

    let mut artifacts: Vec<Artifact> = merged.into_iter().map(|entry| entry.artifact).collect();
    for artifact in &mut artifacts {
        artifact.ecosystems.sort_by_key(|ecosystem| {
            Ecosystem::ALL
                .iter()
                .position(|candidate| candidate == ecosystem)
                .unwrap_or(usize::MAX)
        });
        artifact.ecosystems.dedup();
        if artifact.warnings.is_empty() {
            // keep the field present for the view layer
        }
    }
    artifacts.sort_by(|left, right| {
        type_order(left.artifact_type)
            .cmp(&type_order(right.artifact_type))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.primary_path.cmp(&right.primary_path))
    });

    if artifacts.is_empty() {
        // no-op; an empty result is legitimate
    } else {
        warnings.retain(|warning| !warning.is_empty());
    }

    artifacts
}
