extends Control
## The app shell: header, navigation and the current screen.

const Theme_ := preload("res://theme/elite_theme.gd")
const HeaderBar := preload("res://shell/header_bar.gd")
const MarketScreen := preload("res://screens/market_screen.gd")
const CargoScreen := preload("res://screens/cargo_screen.gd")
const LogScreen := preload("res://screens/log_screen.gd")
const ServicesScreen := preload("res://screens/services_screen.gd")
const GalaxyScreen := preload("res://screens/galaxy_screen.gd")
const EquipmentScreen := preload("res://screens/equipment_screen.gd")
const ShipyardScreen := preload("res://screens/shipyard_screen.gd")
const JobsScreen := preload("res://screens/jobs_screen.gd")
const SavesScreen := preload("res://screens/saves_screen.gd")
const StartScreen := preload("res://screens/start_screen.gd")
const JourneyScreen := preload("res://screens/journey_screen.gd")

var game := EliteGame.new()
var header: PanelContainer
var content := MarginContainer.new()
var screens := {}
var nav_buttons := {}
var start: Control
var playing := false


func _ready() -> void:
	theme = Theme_.make()
	var backdrop := ColorRect.new()
	backdrop.color = Theme_.BACKGROUND
	add_child(backdrop)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 10)

	var margin := MarginContainer.new()
	for side in ["left", "right", "top", "bottom"]:
		margin.add_theme_constant_override("margin_" + side, 12)
	margin.size_flags_vertical = Control.SIZE_EXPAND_FILL
	margin.add_child(root)
	add_child(margin)
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	backdrop.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	get_window().min_size = Vector2i(960, 600)

	header = HeaderBar.new()
	root.add_child(header)

	var body := HBoxContainer.new()
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 12)
	root.add_child(body)

	var nav := VBoxContainer.new()
	nav.custom_minimum_size = Vector2(210, 0)
	nav.add_theme_constant_override("separation", 6)
	body.add_child(nav)

	content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(content)

	_add_screen(nav, "market", "📊  Market", _make_market())
	_add_screen(nav, "cargo", "📦  Cargo", _make(CargoScreen))
	_add_screen(nav, "log", "📜  Trading Log", _make(LogScreen))
	_add_screen(nav, "jobs", "💼  Jobs", _make(JobsScreen))
	_add_screen(nav, "galaxy", "🗺️  Galaxy", _make_galaxy())
	_add_screen(nav, "equipment", "🧰  Equipment", _make_changing(EquipmentScreen))
	_add_screen(nav, "shipyard", "🛠️  Shipyard", _make_changing(ShipyardScreen))
	_add_screen(nav, "services", "🔧  Services", _make_services())
	_add_screen(nav, "saves", "💾  Saves", _make_saves())
	_add_screen(nav, "journey", "🚀  Journey", _make_journey())
	_show("market")
	_refresh_header()

	get_tree().set_auto_accept_quit(false)
	start = StartScreen.new()
	start.setup(game)
	start.game_started.connect(_game_started)
	add_child(start)
	start.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)


func _make_market() -> Control:
	var market := MarketScreen.new()
	market.setup(game)
	market.traded.connect(_refresh_header)
	return market


func _make_saves() -> Control:
	var saves: Control = SavesScreen.new()
	saves.setup(game)
	saves.loaded.connect(_game_started)
	return saves


func _game_started() -> void:
	playing = true
	start.visible = false
	screens["galaxy"].selected = ""
	_refresh_header()
	_show("market")


func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		if playing:
			game.autosave()
		get_tree().quit()


func _make_galaxy() -> Control:
	var galaxy: Control = GalaxyScreen.new()
	galaxy.setup(game)
	galaxy.jump_requested.connect(_begin_journey)
	return galaxy


func _make_journey() -> Control:
	var journey: Control = JourneyScreen.new()
	journey.setup(game)
	journey.arrived.connect(_end_journey)
	return journey


func _begin_journey(destination: String) -> void:
	for key in nav_buttons:
		nav_buttons[key].disabled = true
	_show("journey")
	screens["journey"].start(destination)
	_refresh_header()


func _end_journey() -> void:
	for key in nav_buttons:
		nav_buttons[key].disabled = false
	_refresh_header()
	_show("market")


func _make_changing(script: GDScript) -> Control:
	var screen: Control = script.new()
	screen.setup(game)
	screen.changed.connect(_refresh_header)
	return screen


func _make_services() -> Control:
	var services: Control = ServicesScreen.new()
	services.setup(game)
	services.changed.connect(_refresh_header)
	return services


func _make(script: GDScript) -> Control:
	var screen: Control = script.new()
	screen.setup(game)
	return screen


func _add_screen(nav: VBoxContainer, key: String, title: String, screen: Control) -> void:
	screens[key] = screen
	screen.visible = false
	content.add_child(screen)
	var button := Button.new()
	button.text = title
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.toggle_mode = true
	button.pressed.connect(_show.bind(key))
	nav.add_child(button)
	nav_buttons[key] = button


func _show(key: String) -> void:
	for name in screens:
		screens[name].visible = name == key
		nav_buttons[name].button_pressed = name == key
	if screens[key].has_method("refresh"):
		screens[key].refresh()


func _refresh_header() -> void:
	header.update(game.status())
