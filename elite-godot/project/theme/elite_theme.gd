extends RefCounted
## One theme for the whole game: dark panels, cyan accents, green and red for good and bad.

const BACKGROUND := Color("0a0e14")
const PANEL := Color("111826")
const PANEL_RAISED := Color("182236")
const ACCENT := Color("3de0c8")
const TEXT := Color("d6e2f0")
const MUTED := Color("7d8da3")
const GOOD := Color("5ee08a")
const WARNING := Color("f2c14e")
const BAD := Color("ef6f6c")

## Every font size in the GUI comes from here.
const FONT_SMALL := 14
const FONT_BODY := 17
const FONT_SECTION := 20
const FONT_STRONG := 22
const FONT_TITLE := 26


static func make() -> Theme:
	var theme := Theme.new()
	theme.set_color("font_color", "Label", TEXT)
	theme.set_color("font_color", "Button", TEXT)
	theme.set_color("font_hover_color", "Button", ACCENT)
	theme.set_color("font_pressed_color", "Button", ACCENT)
	theme.set_color("font_disabled_color", "Button", MUTED)
	theme.set_color("font_color", "Tree", TEXT)
	theme.set_color("title_button_color", "Tree", ACCENT)
	theme.set_color("font_selected_color", "Tree", BACKGROUND)
	theme.set_color("font_color", "TabBar", TEXT)
	theme.default_font_size = FONT_BODY
	theme.set_constant("v_separation", "Tree", 8)
	theme.set_constant("item_margin", "Tree", 8)

	theme.set_stylebox("normal", "Button", _box(PANEL_RAISED, 6, ACCENT.darkened(0.5)))
	theme.set_stylebox("hover", "Button", _box(PANEL_RAISED.lightened(0.08), 6, ACCENT))
	theme.set_stylebox("pressed", "Button", _box(ACCENT.darkened(0.6), 6, ACCENT))
	theme.set_stylebox("disabled", "Button", _box(PANEL, 6, MUTED.darkened(0.5)))
	theme.set_stylebox("focus", "Button", StyleBoxEmpty.new())
	theme.set_stylebox("panel", "PanelContainer", _box(PANEL, 8, PANEL_RAISED))
	theme.set_stylebox("panel", "Tree", _box(PANEL, 6, PANEL_RAISED))
	theme.set_stylebox("selected", "Tree", _box(ACCENT, 3, ACCENT))
	theme.set_stylebox("selected_focus", "Tree", _box(ACCENT, 3, ACCENT))
	theme.set_stylebox("title_button_normal", "Tree", _box(PANEL_RAISED, 0, PANEL_RAISED))
	theme.set_stylebox("title_button_hover", "Tree", _box(PANEL_RAISED, 0, PANEL_RAISED))
	theme.set_stylebox("title_button_pressed", "Tree", _box(PANEL_RAISED, 0, PANEL_RAISED))
	theme.set_stylebox("background", "ProgressBar", _box(PANEL_RAISED, 4, PANEL_RAISED))
	theme.set_stylebox("fill", "ProgressBar", _box(GOOD, 4, GOOD))
	return theme


static func _box(fill: Color, radius: int, border: Color) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = fill
	box.set_corner_radius_all(radius)
	box.set_border_width_all(1)
	box.border_color = border
	box.content_margin_left = 10
	box.content_margin_right = 10
	box.content_margin_top = 8
	box.content_margin_bottom = 8
	return box


## Green when the level is healthy, red when low. `full_is_good` false flips it (cargo, power).
static func level_colour(current: int, maximum: int, full_is_good: bool) -> Color:
	var share := float(current) / maxf(float(maximum), 1.0)
	var health := share if full_is_good else 1.0 - share
	if health >= 0.6:
		return GOOD
	if health >= 0.3:
		return WARNING
	return BAD
