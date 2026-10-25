use std::fs;
use std::time::Instant;

use agentdeck_core::roots::Roots;
use agentdeck_core::Manager;
use tempfile::tempdir;

#[test]
#[ignore = "performance benchmark; run with --release --ignored"]
fn scans_ten_thousand_files_within_the_budget() {
    let home = tempdir().unwrap();
    let project = home.path().join("project");
    fs::create_dir_all(project.join(".git")).unwrap();

    let agent = |name: &str| {
        format!("---\nname: {name}\ndescription: Benchmark agent\n---\nBody\n")
    };
    let toml_agent = |name: &str| {
        format!("name = \"{name}\"\ndescription = \"Benchmark agent\"\ndeveloper_instructions = \"Work\"\n")
    };
    let skill = |name: &str| {
        format!("---\nname: {name}\ndescription: Benchmark skill\n---\nBody\n")
    };

    let claude_agents = home.path().join(".claude/agents");
    let project_claude = project.join(".claude/agents");
    let codex_agents = home.path().join(".codex/agents");
    let opencode_agents = project.join(".opencode/agents");
    let agents_skills = home.path().join(".agents/skills");
    for directory in [&claude_agents, &project_claude, &codex_agents, &opencode_agents, &agents_skills] {
        fs::create_dir_all(directory).unwrap();
    }

    for index in 0..2500 {
        fs::write(
            claude_agents.join(format!("claude-{index}.md")),
            agent(&format!("claude-{index}")),
        )
        .unwrap();
        fs::write(
            project_claude.join(format!("project-{index}.md")),
            agent(&format!("project-{index}")),
        )
        .unwrap();
        fs::write(
            codex_agents.join(format!("codex-{index}.toml")),
            toml_agent(&format!("codex-{index}")),
        )
        .unwrap();
        let skill_directory = agents_skills.join(format!("skill-{index}"));
        fs::create_dir_all(&skill_directory).unwrap();
        fs::write(
            skill_directory.join("SKILL.md"),
            skill(&format!("skill-{index}")),
        )
        .unwrap();
    }

    let manager = Manager::new(Roots::new(
        home.path().to_path_buf(),
        home.path().join("appdata"),
        Some(project),
    ));

    let started = Instant::now();
    let result = manager.scan();
    let elapsed = started.elapsed();

    println!(
        "scanned {} artifacts across 10000 files in {:.3}s",
        result.artifacts.len(),
        elapsed.as_secs_f64()
    );
    assert!(result.artifacts.len() >= 7500);
    assert!(
        elapsed.as_secs_f64() < 2.0,
        "full scan took {:.3}s, over the 2s budget",
        elapsed.as_secs_f64()
    );
}
