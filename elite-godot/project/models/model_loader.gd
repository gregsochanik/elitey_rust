extends RefCounted
## Reads ship models from res://models/<id>.json and builds meshes from them.

const DIRECTORY := "res://models/"
const FALLBACK_SHIP := "cobra-mk-iii"


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
	return {"id": id, "vertices": vertices, "edges": edges}


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
