extends RefCounted
## Reads ship models from res://models/<id>.json and builds meshes from them.

const DIRECTORY := "res://models/"
const FALLBACK_SHIP := "cobra-mk-iii"


static func ids() -> Array:
	var found := []
	for file in DirAccess.get_files_at(DIRECTORY):
		if file.ends_with(".json"):
			found.append(file.trim_suffix(".json"))
	found.sort()
	return found


static func load_model(id: String) -> Dictionary:
	var path := DIRECTORY + id + ".json"
	if not FileAccess.file_exists(path):
		return {}
	var data = JSON.parse_string(FileAccess.get_file_as_string(path))
	if typeof(data) != TYPE_DICTIONARY or not data.has("vertices") or not data.has("edges"):
		return {}
	var vertices := PackedVector3Array()
	for vertex in data["vertices"]:
		vertices.append(Vector3(vertex[0], vertex[1], vertex[2]))
	var edges := PackedInt32Array()
	for edge in data["edges"]:
		if edge[0] >= vertices.size() or edge[1] >= vertices.size():
			return {}
		edges.append(edge[0])
		edges.append(edge[1])
	var faces := PackedInt32Array()
	for face in data.get("faces", []):
		if face[0] >= vertices.size() or face[1] >= vertices.size() or face[2] >= vertices.size():
			return {}
		faces.append_array(PackedInt32Array([face[0], face[1], face[2]]))
	var engines := PackedVector3Array()
	for engine in data.get("engines", []):
		engines.append(Vector3(engine[0], engine[1], engine[2]))
	return {"id": id, "vertices": vertices, "edges": edges, "faces": faces, "engines": engines}


## A model's own mesh, or the fallback ship's when no file exists for the id.
static func line_mesh(id: String) -> ArrayMesh:
	var model := load_model(id)
	if model.is_empty():
		model = load_model(FALLBACK_SHIP)
	var points := PackedVector3Array()
	var vertices: PackedVector3Array = model["vertices"]
	for index in model["edges"]:
		points.append(vertices[index])
	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = points
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_LINES, arrays)
	return mesh


## Filled triangles with a normal per face, or null when the model has no faces.
static func solid_mesh(id: String) -> ArrayMesh:
	var model := load_model(id)
	if model.is_empty() or model["faces"].is_empty():
		return null
	var vertices: PackedVector3Array = model["vertices"]
	var points := PackedVector3Array()
	var normals := PackedVector3Array()
	var faces: PackedInt32Array = model["faces"]
	for i in range(0, faces.size(), 3):
		var a := vertices[faces[i]]
		var b := vertices[faces[i + 1]]
		var c := vertices[faces[i + 2]]
		var normal := (b - a).cross(c - a).normalized()
		points.append_array(PackedVector3Array([a, b, c]))
		normals.append_array(PackedVector3Array([normal, normal, normal]))
	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = points
	arrays[Mesh.ARRAY_NORMAL] = normals
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	return mesh
