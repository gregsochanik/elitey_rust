extends VBoxContainer
## What is fitted and what is for sale.

signal changed

const Theme_ := preload("res://theme/elite_theme.gd")

var game: EliteGame
var heading := Label.new()
var power := Label.new()
var installed_tree := Tree.new()
var shop_tree := Tree.new()
var buy_button := Button.new()
var sell_button := Button.new()
var feedback := Label.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	add_theme_constant_override("separation", 8)
	heading.text = "🧰  Equipment"
	heading.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	heading.add_theme_color_override("font_color", Theme_.ACCENT)
	add_child(heading)
	add_child(power)

	add_child(_title("Installed"))
	_prepare(installed_tree, ["Category", "Slots", "Item", "Power", "Effect", "Refund"])
	installed_tree.item_selected.connect(_update_buttons)
	add_child(installed_tree)

	add_child(_title("For sale"))
	_prepare(shop_tree, ["Category", "Item", "Quality", "Price", "Power", "Effect", ""])
	shop_tree.item_selected.connect(_update_buttons)
	add_child(shop_tree)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	buy_button.text = "Buy and install"
	buy_button.pressed.connect(func(): _act(game.buy_equipment(_selected_id(shop_tree))))
	row.add_child(buy_button)
	sell_button.text = "Sell installed"
	sell_button.pressed.connect(func(): _act(game.sell_equipment(_selected_id(installed_tree))))
	row.add_child(sell_button)
	add_child(row)
	add_child(feedback)
	refresh()


func _title(text: String) -> Label:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", Theme_.FONT_SECTION)
	return label


func _prepare(tree: Tree, titles: Array) -> void:
	tree.columns = titles.size()
	tree.column_titles_visible = true
	tree.hide_root = true
	tree.select_mode = Tree.SELECT_ROW
	tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	for i in titles.size():
		tree.set_column_title(i, titles[i])
		tree.set_column_expand(i, true)
		tree.set_column_expand_ratio(i, 4 if titles[i] == "Effect" else 2)


func refresh() -> void:
	var fitted: Dictionary = game.installed_equipment()
	var shop: Dictionary = game.equipment_shop()
	var free: int = fitted["power_capacity"] - fitted["power_used"]
	power.text = "⚡ Power %d/%d used, %d free     📦 Cargo capacity %d t     💰 %d cr" % [
		fitted["power_used"], fitted["power_capacity"], free, fitted["cargo_capacity"], shop["credits"]
	]

	var keep_installed := _selected_id(installed_tree)
	var keep_shop := _selected_id(shop_tree)
	installed_tree.clear()
	var root := installed_tree.create_item()
	var last := ""
	for entry in fitted["items"]:
		var item := installed_tree.create_item(root)
		item.set_metadata(0, entry["id"])
		item.set_text(0, entry["category"] if entry["category"] != last else "")
		last = entry["category"]
		item.set_text(1, "%d/%d" % [entry["slots_used"], entry["slots_capacity"]])
		item.set_text(2, entry["name"])
		item.set_text(3, _power_text(entry))
		item.set_text(4, entry["effect"])
		item.set_text(5, "%d cr" % entry["refund"])
		for column in [1, 3, 5]:
			item.set_text_alignment(column, HORIZONTAL_ALIGNMENT_RIGHT)
		if entry["id"] == keep_installed:
			item.select(0)

	shop_tree.clear()
	root = shop_tree.create_item()
	last = ""
	for entry in shop["items"]:
		var item := shop_tree.create_item(root)
		item.set_metadata(0, entry["id"])
		item.set_text(0, entry["category"] if entry["category"] != last else "")
		last = entry["category"]
		item.set_text(1, entry["name"])
		item.set_text(2, entry["quality"])
		item.set_text(3, "%d cr" % entry["price"])
		item.set_text(4, _power_text(entry))
		item.set_text(5, entry["effect"])
		for column in [3, 4]:
			item.set_text_alignment(column, HORIZONTAL_ALIGNMENT_RIGHT)
		if entry["installed"]:
			item.set_text(6, "installed")
			item.set_custom_color(6, Theme_.GOOD)
		elif entry["price"] > shop["credits"]:
			item.set_custom_color(3, Theme_.BAD)
		if entry["id"] == keep_shop:
			item.select(0)
	_update_buttons()


func _power_text(entry: Dictionary) -> String:
	return "+%d" % entry["output"] if entry["output"] > 0 else "%d" % entry["power"]


func _selected_id(tree: Tree) -> String:
	var item := tree.get_selected()
	return item.get_metadata(0) if item else ""


func _update_buttons() -> void:
	buy_button.disabled = _selected_id(shop_tree) == ""
	sell_button.disabled = _selected_id(installed_tree) == ""


func _act(result: Dictionary) -> void:
	feedback.text = ("✅  " if result["ok"] else "❌  ") + result["message"]
	feedback.add_theme_color_override("font_color", Theme_.GOOD if result["ok"] else Theme_.BAD)
	refresh()
	changed.emit()
