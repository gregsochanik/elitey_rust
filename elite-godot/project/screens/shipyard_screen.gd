extends VBoxContainer
## Ships for sale. Buying one sells the fitted equipment at half price.

signal changed

const Theme_ := preload("res://theme/elite_theme.gd")
const ModelViewer := preload("res://widgets/model_viewer.gd")

var game: EliteGame
var heading := Label.new()
var summary := Label.new()
var tree := Tree.new()
var viewer := ModelViewer.new()
var buy_button := Button.new()
var confirm := ConfirmationDialog.new()
var feedback := Label.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	add_theme_constant_override("separation", 10)
	heading.text = "🚀  Shipyard"
	heading.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	heading.add_theme_color_override("font_color", Theme_.ACCENT)
	add_child(heading)
	add_child(summary)

	tree.columns = 7
	tree.column_titles_visible = true
	tree.hide_root = true
	tree.select_mode = Tree.SELECT_ROW
	tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var titles := ["Ship", "Cargo", "Fuel", "Hull", "Cost", "Upgrade cost", ""]
	for i in titles.size():
		tree.set_column_title(i, titles[i])
		tree.set_column_expand(i, true)
		tree.set_column_expand_ratio(i, 3 if i == 0 or i == 6 else 2)
	tree.item_selected.connect(_update_button)
	var yard_row := HBoxContainer.new()
	yard_row.add_theme_constant_override("separation", 12)
	yard_row.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tree.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	yard_row.add_child(tree)
	viewer.custom_minimum_size = Vector2(320, 0)
	viewer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	viewer.size_flags_stretch_ratio = 0.8
	yard_row.add_child(viewer)
	add_child(yard_row)

	buy_button.text = "Buy selected ship"
	buy_button.pressed.connect(_ask)
	add_child(buy_button)
	add_child(feedback)

	confirm.title = "Buy ship"
	confirm.confirmed.connect(_buy)
	add_child(confirm)
	refresh()


func refresh() -> void:
	var yard: Dictionary = game.shipyard()
	summary.text = "Current ship: %s     💰 %d cr" % [yard["current"], yard["credits"]]
	var keep := _selected_id()
	tree.clear()
	var root := tree.create_item()
	for ship in yard["ships"]:
		var item := tree.create_item(root)
		item.set_metadata(0, ship["id"])
		item.set_metadata(1, ship)
		item.set_text(0, ship["name"])
		item.set_text(1, "%d t" % ship["cargo"])
		item.set_text(2, "%d ly" % ship["fuel"])
		item.set_text(3, "%d" % ship["hull"])
		item.set_text(4, "%d cr" % ship["cost"])
		for column in range(1, 6):
			item.set_text_alignment(column, HORIZONTAL_ALIGNMENT_RIGHT)
		if ship["is_current"]:
			item.set_text(6, "current")
			item.set_custom_color(6, Theme_.GOOD)
			item.set_custom_color(0, Theme_.GOOD)
		elif ship["is_upgrade"]:
			item.set_text(5, "%d cr" % ship["upgrade_cost"])
			item.set_custom_color(5, Theme_.GOOD if ship["affordable"] else Theme_.BAD)
			if not ship["affordable"]:
				item.set_text(6, "can't afford")
				item.set_custom_color(6, Theme_.BAD)
		else:
			item.set_custom_color(0, Theme_.MUTED)
		if ship["id"] == keep:
			item.select(0)
	_update_button()


func _selected_id() -> String:
	var item := tree.get_selected()
	return item.get_metadata(0) if item else ""


func _update_button() -> void:
	var item := tree.get_selected()
	viewer.show_model(_selected_id())
	buy_button.disabled = item == null or not item.get_metadata(1)["is_upgrade"]


func _ask() -> void:
	var ship: Dictionary = tree.get_selected().get_metadata(1)
	confirm.dialog_text = "Buy a %s for %d cr?\nYour fitted equipment is sold at half price and the loadout resets." % [ship["name"], ship["upgrade_cost"]]
	confirm.popup_centered()


func _buy() -> void:
	var result: Dictionary = game.buy_ship(_selected_id())
	feedback.text = ("✅  " if result["ok"] else "❌  ") + result["message"]
	feedback.add_theme_color_override("font_color", Theme_.GOOD if result["ok"] else Theme_.BAD)
	refresh()
	changed.emit()
