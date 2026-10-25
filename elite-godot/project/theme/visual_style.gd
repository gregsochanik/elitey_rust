extends RefCounted
## How 3D things are drawn. Models and the game stay the same; the style decides colours, glow and what node a model id becomes.

const ModelLoader := preload("res://models/model_loader.gd")
const Events := preload("res://theme/visual_style_events.gd")

const PREFERENCES := "user://preferences.cfg"
const DEFAULT := "wireframe"

const STYLES := {
	"wireframe": {
		"colours": {
			"player": Color(0.35, 1.0, 0.85),
			"enemy": Color(1.0, 0.35, 0.3),
			"station": Color(0.35, 1.0, 0.85),
			"stars": Color(0.8, 0.9, 1.0),
			"beam_player": Color(0.5, 1.0, 0.9),
			"beam_enemy": Color(1.0, 0.45, 0.3),
			"shield": Color(0.4, 0.7, 1.0),
			"explosion": Color(1.0, 0.7, 0.3),
		},
		"glow_intensity": 2.0,
		"glow_bloom": 0.15,
	},
}

static var events := Events.new()
static var current_name := ""


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


static func colour(role: String) -> Color:
	return _settings()["colours"][role]


static func environment() -> Environment:
	var settings := _settings()
	var environment := Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = Color.BLACK
	environment.glow_enabled = true
	environment.glow_intensity = settings["glow_intensity"]
	environment.glow_bloom = settings["glow_bloom"]
	return environment


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
	node.mesh = ModelLoader.line_mesh(model_id)
	node.material_override = material(role)
	return node


## Redraw an existing node after the style changes.
static func restyle(node: MeshInstance3D, model_id: String, role: String) -> void:
	node.mesh = ModelLoader.line_mesh(model_id)
	node.material_override = material(role)


static func _settings() -> Dictionary:
	return STYLES[current()]


static func _saved_name() -> String:
	var config := ConfigFile.new()
	if config.load(PREFERENCES) != OK:
		return DEFAULT
	var name: String = config.get_value("visual", "style", DEFAULT)
	return name if STYLES.has(name) else DEFAULT
