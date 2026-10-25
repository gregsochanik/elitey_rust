extends Control
## The app shell: header, navigation and the current screen.

const Theme_ := preload("res://theme/elite_theme.gd")
const HeaderBar := preload("res://shell/header_bar.gd")
const MarketScreen := preload("res://screens/market_screen.gd")
const JourneyScreen := preload("res://screens/journey_screen.gd")

var game := EliteGame.new()
var header: PanelContainer
var content := MarginContainer.new()
var screens := {}
var nav_buttons := {}


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
	_add_screen(nav, "journey", "🚀  Journey", JourneyScreen.new())
	_show("market")
	_refresh_header()


func _make_market() -> Control:
	var market := MarketScreen.new()
	market.setup(game)
	market.traded.connect(_refresh_header)
	return market


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


func _refresh_header() -> void:
	header.update(game.status())
