use super::*;

#[test]
fn add_pattern_empty_file() {
    assert_eq!(
        add_pattern("", "motif").unwrap(),
        "motif\n".to_string(),
        "Failed to add to empty content"
    );
}

#[test]
fn add_pattern_test() {
    assert_eq!(
        add_pattern("a\nb", "motif\n").unwrap(),
        "a\nb\nmotif\n".to_string(),
        "Failed to add \\n to content"
    );
}

#[test]
fn add_pattern_double() {
    let motif = "motif";
    assert_eq!(
        add_pattern(motif, motif).unwrap_err(),
        format!("Motif {} already in .syncignore", motif),
        "Unable to detect double"
    );
}

#[test]
fn add_pattern_invalid_pattern() {
    assert!(
        add_pattern("", "{a,b").is_err(),
        "Unable to detect invalid motif"
    );
}

#[test]
fn add_pattern_empty_or_comment_add() {
    assert_eq!(
        add_pattern("", "").unwrap_err(),
        "Cannot add empty motif".to_string(),
        "Unable to detect empty motif"
    );

    assert_eq!(
        add_pattern("", "#Comment").unwrap_err(),
        "Cannot add commentary".to_string(),
        "Unable to detect commentary"
    );
}

#[test]
fn remove_pattern_comments_empty_motif() {
    let base = "#comment\n \n*.blend1\n";
    assert_eq!(
        remove_pattern(base, "*.blend1").unwrap(),
        "#comment\n \n",
        "Unable to conserve content"
    );
}

#[test]
fn remove_pattern_empty() {
    let base = "*.blend1\n";
    assert_eq!(
        remove_pattern(base, "").unwrap_err(),
        "Cannot remove empty motif".to_string(),
        "Unable to detect empty motif"
    );
}

#[test]
fn remove_pattern_test() {
    let base = "*.blend1";
    assert_eq!(
        remove_pattern(base, "*.blend1").unwrap(),
        "\n",
        "Unable to remove motif"
    );
}

#[test]
fn remove_pattern_perfect_match() {
    assert!(
        remove_pattern("#comment\n*.blend1\nThumbs.db\n", "blend1").is_err(),
        "Failed to avoid remove blend1"
    );
}

#[test]
fn remove_patter_remove_empty() {
    assert!(
        remove_pattern("a\n\n", "zzz").is_err(),
        "Should not remove anything"
    );
}

// ---------------------------------------------------------------------------
// Motifs .syncignore : validité, ligne entière, espaces
// ---------------------------------------------------------------------------

#[test]
fn add_pattern_validity() {
    // (motif, doit être accepté ?)
    let cases = [
        ("{a,b", false),  // accolade non fermée
        ("[z-a]", false), // intervalle inversé
        ("a\\", false),   // '\' en fin de motif
        ("[abc", true),   // accepté par la crate : '[' non fermé = caractère normal
    ];
    for (motif, accepted) in cases {
        assert_eq!(
            add_pattern("", motif).is_ok(),
            accepted,
            "Wrong validation for motif {}",
            motif
        );
    }
}

#[test]
fn add_pattern_longer_pattern_is_not_duplicate() {
    assert_eq!(
        add_pattern("*.blend1\n", "blend1").unwrap(),
        "*.blend1\nblend1\n",
        "'blend1' is not a duplicate of '*.blend1'"
    );
}

#[test]
fn remove_pattern_single_letter_removes_nothing() {
    assert!(
        remove_pattern("#commentaire\nThumbs.db\n", "a").is_err(),
        "'a' must not remove lines that only contain the letter a"
    );
}

#[test]
fn add_pattern_duplicate_with_spaces_in_file() {
    assert!(
        add_pattern("  *.blend1  \n", "*.blend1").is_err(),
        "Duplicate not detected when the file line has spaces"
    );
}

#[test]
fn remove_pattern_with_spaces_in_argument() {
    assert_eq!(
        remove_pattern("*.blend1\nThumbs.db\n", "  *.blend1  ").unwrap(),
        "Thumbs.db\n",
        "Motif with spaces around was not removed"
    );
}

// ---------------------------------------------------------------------------
// open_file : BOM et fichier absent
// ---------------------------------------------------------------------------

#[test]
fn open_file_strips_bom() {
    let dir = tempfile::tempdir().expect("Unable to create temp dir");
    let path = dir.path().join(".syncignore");
    std::fs::write(&path, "\u{feff}*.blend1\n").expect("Unable to write file");

    assert_eq!(
        open_file(&path).unwrap(),
        "*.blend1\n",
        "BOM was not removed"
    );
}

#[test]
fn open_file_missing_gives_empty_content() {
    let dir = tempfile::tempdir().expect("Unable to create temp dir");
    let path = dir.path().join("absent");

    assert_eq!(
        open_file(&path).unwrap(),
        "",
        "A missing file must be read as empty"
    );
}

// ---------------------------------------------------------------------------
// .filesync : add_folder / remove_folder
// ---------------------------------------------------------------------------

/// Début de chemin absolu valide sur la plateforme qui exécute les tests.
#[cfg(windows)]
const ROOT: &str = "C:\\";
#[cfg(unix)]
const ROOT: &str = "/home/";

/// Chemin machine absolu, quelle que soit la plateforme.
fn abs(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}{}", ROOT, name))
}

/// Ligne .filesync écrite à la main, comme le ferait l'utilisateur.
fn sync_line(machine: &str, disk_folder: &str, machine_folder: &str) -> String {
    format!(
        "{} -> {} -> {}{}",
        machine, disk_folder, ROOT, machine_folder
    )
}

/// Les configurations valides d'un contenu (commentaires, lignes vides et
/// lignes invalides ignorés). Ne dépend pas de la mise en forme exacte.
fn valid_configs(content: &str) -> Vec<crate::config::SyncConfig> {
    content
        .lines()
        .filter_map(|line| parse_line(line).ok())
        .collect()
}

#[test]
fn add_folder_to_empty_content() {
    let result = add_folder("", "HAL", Path::new("Projets"), &abs("Projets")).unwrap();

    let configs = valid_configs(&result);
    assert_eq!(
        configs.len(),
        1,
        "Exactly one line expected in {:?}",
        result
    );
    assert!(configs[0].matches("HAL", Path::new("Projets")));
    assert_eq!(configs[0].machine_path, abs("Projets"));
    assert!(result.ends_with('\n'), "Content must end with a new line");
}

#[test]
fn add_folder_keeps_existing_content() {
    let content = format!("# commentaire\n\n{}\n", sync_line("AUTRE-PC", "X", "X"));
    let result = add_folder(&content, "HAL", Path::new("Projets"), &abs("Projets")).unwrap();

    assert!(
        result.starts_with(&content),
        "Existing content was modified : {:?}",
        result
    );
    assert_eq!(valid_configs(&result).len(), 2);
}

#[test]
fn add_folder_without_final_new_line() {
    // Le dernier '\n' manque : la nouvelle ligne ne doit pas s'y coller.
    let content = sync_line("AUTRE-PC", "X", "X");
    let result = add_folder(&content, "HAL", Path::new("Projets"), &abs("Projets")).unwrap();

    assert_eq!(
        valid_configs(&result).len(),
        2,
        "New line was glued to the previous one : {:?}",
        result
    );
}

#[test]
fn add_folder_duplicate_ignores_case_and_machine_path() {
    // Même machine (casse différente) et même dossier disque, autre chemin machine.
    let content = format!("{}\n", sync_line("HAL", "Projets", "A"));

    assert!(
        add_folder(&content, "hal", Path::new("Projets"), &abs("B")).is_err(),
        "Same machine + same disk folder must be a duplicate"
    );
}

#[test]
fn add_folder_same_machine_other_disk_folder() {
    let content = format!("{}\n", sync_line("HAL", "Projets", "Projets"));

    assert!(add_folder(&content, "HAL", Path::new("Images"), &abs("Images")).is_ok());
}

#[test]
fn add_folder_other_machine_same_disk_folder() {
    let content = format!("{}\n", sync_line("HAL", "Projets", "Projets"));

    assert!(add_folder(&content, "AUTRE-PC", Path::new("Projets"), &abs("Projets")).is_ok());
}

#[test]
fn add_folder_uses_parse_line_validation() {
    // Chemin disque absolu
    assert!(
        add_folder("", "HAL", &abs("Projets"), &abs("Projets"))
            .unwrap_err()
            .contains("relative")
    );
    // Chemin machine relatif
    assert!(
        add_folder("", "HAL", Path::new("Projets"), Path::new("Projets"))
            .unwrap_err()
            .contains("absolute")
    );
    // Séparateur dans un chemin
    assert!(add_folder("", "HAL", Path::new("a->b"), &abs("Projets")).is_err());
}

#[test]
fn remove_folder_only_removes_target_line() {
    let content = format!(
        "# commentaire\n{}\n{}\n",
        sync_line("HAL", "Projets", "Projets"),
        sync_line("AUTRE-PC", "Projets", "Projets")
    );
    let result = remove_folder(&content, "HAL", Path::new("Projets")).unwrap();

    assert!(result.contains("# commentaire"), "Comment was removed");
    let configs = valid_configs(&result);
    assert_eq!(configs.len(), 1);
    assert!(
        configs[0].matches("AUTRE-PC", Path::new("Projets")),
        "The other machine's line was removed"
    );
}

#[test]
fn remove_folder_ignores_case() {
    let content = format!("{}\n", sync_line("hal", "Projets", "Projets"));

    assert!(remove_folder(&content, "HAL", Path::new("Projets")).is_ok());
}

#[test]
fn remove_folder_trailing_slash() {
    // Comparaison de Path : "Projets/" et "Projets" sont le même dossier.
    let content = format!("{}\n", sync_line("HAL", "Projets", "Projets"));

    assert!(remove_folder(&content, "HAL", Path::new("Projets/")).is_ok());
}

#[test]
fn remove_folder_missing_is_error() {
    let content = format!("{}\n", sync_line("AUTRE-PC", "Projets", "Projets"));

    // Le dossier n'existe pas du tout
    assert!(remove_folder(&content, "HAL", Path::new("Images")).is_err());
    // Le dossier existe, mais pour une autre machine
    assert!(remove_folder(&content, "HAL", Path::new("Projets")).is_err());
}

#[test]
fn remove_folder_keeps_invalid_lines() {
    let content = format!(
        "ligne invalide\n{}\n",
        sync_line("HAL", "Projets", "Projets")
    );
    let result = remove_folder(&content, "HAL", Path::new("Projets")).unwrap();

    assert!(
        result.contains("ligne invalide"),
        "Invalid line was removed"
    );
}
