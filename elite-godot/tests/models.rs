use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn models_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("project/models")
}

fn indices(value: &Value, key: &str, per_item: usize) -> Result<Vec<Vec<usize>>, String> {
    let Some(items) = value.get(key) else {
        return Ok(Vec::new());
    };
    items
        .as_array()
        .ok_or(format!("{key} is not a list"))?
        .iter()
        .map(|item| {
            let list = item
                .as_array()
                .ok_or(format!("{key} entry is not a list"))?;
            if list.len() != per_item {
                return Err(format!("{key} entry {item} needs {per_item} indices"));
            }
            list.iter()
                .map(|n| {
                    n.as_u64()
                        .map(|n| n as usize)
                        .ok_or(format!("{key} entry {item} has a non-index"))
                })
                .collect()
        })
        .collect()
}

fn check(id: &str, model: &Value) -> Result<(), String> {
    if model["id"] != id {
        return Err(format!("id {} does not match the file name", model["id"]));
    }
    let vertices = model["vertices"]
        .as_array()
        .filter(|v| !v.is_empty())
        .ok_or("vertices are missing or empty")?;
    for vertex in vertices {
        let coords = vertex.as_array().ok_or("a vertex is not a list")?;
        if coords.len() != 3 || coords.iter().any(|c| c.as_f64().is_none()) {
            return Err(format!("vertex {vertex} needs three numbers"));
        }
    }

    let edges = indices(model, "edges", 2)?;
    if edges.is_empty() {
        return Err("a model needs edges".to_string());
    }
    let faces = indices(model, "faces", 3)?;

    let mut used = HashSet::new();
    let mut seen_edges = HashSet::new();
    for edge in &edges {
        if edge.iter().any(|&i| i >= vertices.len()) {
            return Err(format!("edge {edge:?} points past the last vertex"));
        }
        if edge[0] == edge[1] {
            return Err(format!("edge {edge:?} joins a vertex to itself"));
        }
        if !seen_edges.insert((edge[0].min(edge[1]), edge[0].max(edge[1]))) {
            return Err(format!("edge {edge:?} is listed twice"));
        }
        used.extend(edge.iter().copied());
    }
    for face in &faces {
        if face.iter().any(|&i| i >= vertices.len()) {
            return Err(format!("face {face:?} points past the last vertex"));
        }
        if face[0] == face[1] || face[1] == face[2] || face[0] == face[2] {
            return Err(format!("face {face:?} is degenerate"));
        }
        used.extend(face.iter().copied());
    }
    if let Some(unused) = (0..vertices.len()).find(|i| !used.contains(i)) {
        return Err(format!("vertex {unused} is not used by any edge or face"));
    }
    if let Some(uvs) = model.get("uvs") {
        if uvs.as_array().map(Vec::len) != Some(vertices.len()) {
            return Err("uvs need one entry per vertex".to_string());
        }
    }
    Ok(())
}

#[test]
fn every_model_file_is_well_formed() {
    let mut checked = 0;
    for entry in fs::read_dir(models_directory()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let id = path.file_stem().unwrap().to_string_lossy().to_string();
        let model: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap())
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        check(&id, &model).unwrap_or_else(|error| panic!("{id}: {error}"));
        checked += 1;
    }
    assert!(checked >= 6, "only {checked} models found");
}

#[test]
fn every_ship_has_a_model_and_so_do_the_raider_and_station() {
    let ships: Value = serde_json::from_str(
        &fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../elite-core/data/ships.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut ids: Vec<String> = ships
        .as_array()
        .unwrap()
        .iter()
        .map(|ship| ship["id"].as_str().unwrap().to_string())
        .collect();
    ids.extend(["raider".to_string(), "station".to_string()]);
    for id in ids {
        assert!(
            models_directory().join(format!("{id}.json")).exists(),
            "no model for {id}"
        );
    }
}

#[test]
fn the_check_rejects_broken_models() {
    let model = |json: &str| serde_json::from_str::<Value>(json).unwrap();
    let ok = r#"{"id":"m","vertices":[[0,0,0],[1,0,0],[0,1,0]],"edges":[[0,1],[1,2]]}"#;
    assert!(check("m", &model(ok)).is_ok());

    let cases = [
        (
            r#"{"id":"m","vertices":[[0,0,0],[1,0,0]],"edges":[[0,2]]}"#,
            "past the last",
        ),
        (
            r#"{"id":"m","vertices":[[0,0,0],[1,0,0]],"edges":[[1,1]]}"#,
            "to itself",
        ),
        (
            r#"{"id":"m","vertices":[[0,0,0],[1,0,0]],"edges":[[0,1],[1,0]]}"#,
            "twice",
        ),
        (
            r#"{"id":"m","vertices":[[0,0,0],[1,0,0],[5,5,5]],"edges":[[0,1]]}"#,
            "not used",
        ),
        (
            r#"{"id":"m","vertices":[[0,0,0],[1,0,0],[0,1,0]],"edges":[[0,1]],"faces":[[0,1,1]]}"#,
            "degenerate",
        ),
        (
            r#"{"id":"x","vertices":[[0,0,0],[1,0,0]],"edges":[[0,1]]}"#,
            "file name",
        ),
        (
            r#"{"id":"m","vertices":[[0,0],[1,0,0]],"edges":[[0,1]]}"#,
            "three numbers",
        ),
        (
            r#"{"id":"m","vertices":[[0,0,0],[1,0,0]],"edges":[[0,1]],"uvs":[[0,0]]}"#,
            "uvs",
        ),
    ];
    for (json, expected) in cases {
        let error = check("m", &model(json)).unwrap_err();
        assert!(error.contains(expected), "{json}: {error}");
    }
}
