extends RefCounted
## How 3D things are drawn. Models and the game stay the same; the style decides colours, glow, light and what node a model id becomes.

const ModelLoader := preload("res://models/model_loader.gd")
const Events := preload("res://theme/visual_style_events.gd")

const PREFERENCES := "user://preferences.cfg"
const DEFAULT := "wireframe"

const COLOURS := {
	"player": Color(0.35, 1.0, 0.85),
	"enemy": Color(1.0, 0.35, 0.3),
	"station": Color(0.35, 1.0, 0.85),
	"stars": Color(0.8, 0.9, 1.0),
	"beam_player": Color(0.5, 1.0, 0.9),
	"beam_enemy": Color(1.0, 0.45, 0.3),
	"shield": Color(0.4, 0.7, 1.0),
	"explosion": Color(1.0, 0.7, 0.3),
}

## `draws` is how a model is drawn: lines, lines over a black fill that hides the far edges, or a lit fill.
const STYLES := {
	"wireframe": {"draws": "lines", "glow_intensity": 2.0, "glow_bloom": 0.15},
	"hidden-line": {"draws": "hidden_line", "glow_intensity": 2.0, "glow_bloom": 0.15},
	"flat": {"draws": "flat", "glow_intensity": 0.4, "glow_bloom": 0.05, "ambient": Color(0.22, 0.27, 0.34)},
}

## Key light colour and strength by spectral class letter
const STAR_LIGHT := {
	"O": [Color(0.7, 0.8, 1.0), 1.7],
	"B": [Color(0.75, 0.85, 1.0), 1.6],
	"A": [Color(0.95, 0.97, 1.0), 1.5],
	"F": [Color(1.0, 0.97, 0.88), 1.4],
	"G": [Color(1.0, 0.93, 0.78), 1.3],
	"K": [Color(1.0, 0.8, 0.55), 1.1],
	"M": [Color(1.0, 0.6, 0.4), 0.9],
	"D": [Color(0.9, 0.95, 1.0), 0.8],
	"L": [Color(0.9, 0.4, 0.3), 0.5],
}
const DEFAULT_STAR := "G"

const HIDDEN_LINE_FILL_SCALE := 0.994

static var events := Events.new()
static var current_name := ""
static var star_letter := DEFAULT_STAR


static func current() -> String:
	if current_name == "":
		current_name = _saved_name()
	return current_name


static func names() -> Array:
	return STYLES.keys()


## Switch style; the choice is a preference, kept apart from saved games.
static func use(name: String) -> void:
	if not STYLES.has(name) or name == current():
		return
	current_name = name
	var config := ConfigFile.new()
	config.load(PREFERENCES)
	config.set_value("visual", "style", name)
	config.save(PREFERENCES)
	events.changed.emit()


## Light ships by the star of the system the commander is in; `code` is a class such as "M3V".
static func set_star(code: String) -> void:
	var letter := code.substr(0, 1).to_upper()
	if not STAR_LIGHT.has(letter):
		letter = DEFAULT_STAR
	if letter == star_letter:
		return
	star_letter = letter
	events.changed.emit()


static func colour(role: String) -> Color:
	return COLOURS[role]


static func environment() -> Environment:
	var settings := _settings()
	var environment := Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = Color.BLACK
	environment.glow_enabled = true
	environment.glow_intensity = settings["glow_intensity"]
	environment.glow_bloom = settings["glow_bloom"]
	if settings.has("ambient"):
		environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
		environment.ambient_light_color = settings["ambient"]
	return environment


## Lights the style needs in a scene; none for unlit styles. Add them to the scene that draws models.
static func lights() -> Array:
	if _settings()["draws"] != "flat":
		return []
	var star: Array = STAR_LIGHT[star_letter]
	var key := DirectionalLight3D.new()
	key.rotation_degrees = Vector3(-40, -35, 0)
	key.light_color = star[0]
	key.light_energy = star[1]
	var fill := DirectionalLight3D.new()
	fill.rotation_degrees = Vector3(-10, 145, 0)
	fill.light_energy = 0.3
	fill.light_color = Color(0.7, 0.8, 1.0)
	return [key, fill]


static func material(role: String, fading := false) -> StandardMaterial3D:
	var material := StandardMaterial3D.new()
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.albedo_color = colour(role)
	if fading:
		material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	return material


## The node that draws a model id in the current style.
static func model_node(model_id: String, role: String) -> MeshInstance3D:
	var node := MeshInstance3D.new()
	restyle(node, model_id, role)
	return node


## Redraw an existing node after the style changes. A style that needs faces falls back to lines for a model without them.
static func restyle(node: MeshInstance3D, model_id: String, role: String) -> void:
	for child in node.get_children():
		child.queue_free()
	var draws: String = _settings()["draws"]
	var solid: ArrayMesh = ModelLoader.solid_mesh(model_id) if draws != "lines" else null
	if solid == null:
		draws = "lines"
	node.mesh = ModelLoader.line_mesh(model_id)
	node.material_override = material(role)
	_add_engine_glow(node, model_id, role)
	match draws:
		"hidden_line":
			var fill := MeshInstance3D.new()
			fill.mesh = solid
			fill.scale = Vector3.ONE * HIDDEN_LINE_FILL_SCALE
			fill.material_override = _fill_material(Color.BLACK, true)
			node.add_child(fill)
		"flat":
			node.mesh = solid
			node.material_override = _fill_material(colour(role).lerp(Color(0.75, 0.78, 0.8), 0.55).darkened(0.25), false)


## A small bright sphere at each engine; the scene pulses anything in the `engine_glow` group.
static func _add_engine_glow(node: MeshInstance3D, model_id: String, role: String) -> void:
	var model := ModelLoader.load_model(model_id)
	if model.is_empty():
		return
	var glow_colour := colour(role).lerp(Color.WHITE, 0.55)
	for point in model["engines"]:
		var glow := MeshInstance3D.new()
		var sphere := SphereMesh.new()
		sphere.radius = 0.11
		sphere.height = 0.22
		glow.mesh = sphere
		var glow_material := StandardMaterial3D.new()
		glow_material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		glow_material.albedo_color = glow_colour
		glow.material_override = glow_material
		glow.position = point
		glow.add_to_group("engine_glow")
		node.add_child(glow)


static func _fill_material(colour: Color, unshaded: bool) -> StandardMaterial3D:
	var material := StandardMaterial3D.new()
	material.cull_mode = BaseMaterial3D.CULL_DISABLED
	material.albedo_color = colour
	if unshaded:
		material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	else:
		material.roughness = 0.8
	return material


static func _settings() -> Dictionary:
	return STYLES[current()]


static func _saved_name() -> String:
	var config := ConfigFile.new()
	if config.load(PREFERENCES) != OK:
		return DEFAULT
	var name: String = config.get_value("visual", "style", DEFAULT)
	return name if STYLES.has(name) else DEFAULT
