extends PanelContainer
## Always-visible commander and ship summary.

const Theme_ := preload("res://theme/elite_theme.gd")

var identity := Label.new()
var credits := Label.new()
var location := Label.new()
var standing := Label.new()
var bars := {}


func _ready() -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 24)
	add_child(row)

	identity.add_theme_font_size_override("font_size", 24)
	row.add_child(identity)
	row.add_child(location)
	credits.add_theme_color_override("font_color", Theme_.GOOD)
	row.add_child(credits)
	row.add_child(standing)

	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(spacer)

	for key in ["fuel", "hull", "cargo", "power"]:
		row.add_child(_bar(key))


func _bar(key: String) -> Control:
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 2)
	var label := Label.new()
	label.add_theme_font_size_override("font_size", 16)
	label.add_theme_color_override("font_color", Theme_.MUTED)
	box.add_child(label)
	var bar := ProgressBar.new()
	bar.show_percentage = false
	bar.custom_minimum_size = Vector2(130, 12)
	bar.add_theme_stylebox_override("fill", StyleBoxFlat.new())
	box.add_child(bar)
	bars[key] = {"label": label, "bar": bar}
	return box


func update(status: Dictionary) -> void:
	identity.text = "🚀  %s — %s" % [status["commander_name"], status["ship_name"]]
	location.text = "📍 %s" % status["location"]
	credits.text = "💰 %d cr" % status["credits"]
	standing.text = "⭐ %s · ⚔️ %s" % [status["standing"], status["combat_rating"]]
	_set_bar("fuel", "⛽ Fuel", status["fuel_current"], status["fuel_capacity"], true)
	_set_bar("hull", "🛡️ Hull", status["hull"], status["max_hull"], true)
	_set_bar("cargo", "📦 Cargo", status["cargo_used"], status["cargo_capacity"], false)
	_set_bar("power", "⚡ Power", status["power_used"], status["power_capacity"], false)


func _set_bar(key: String, title: String, current: int, maximum: int, full_is_good: bool) -> void:
	var entry: Dictionary = bars[key]
	entry["label"].text = "%s  %d/%d" % [title, current, maximum]
	var bar: ProgressBar = entry["bar"]
	bar.max_value = maximum
	bar.value = current
	var fill := StyleBoxFlat.new()
	fill.bg_color = Theme_.level_colour(current, maximum, full_is_good)
	fill.set_corner_radius_all(4)
	bar.add_theme_stylebox_override("fill", fill)
