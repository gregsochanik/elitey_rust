extends VBoxContainer
## Buy and sell at the current system's market.

signal traded

const Theme_ := preload("res://theme/elite_theme.gd")

var game: EliteGame
var heading := Label.new()
var tree := Tree.new()
var quantity := SpinBox.new()
var feedback := Label.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	add_theme_constant_override("separation", 10)
	heading.add_theme_font_size_override("font_size", 30)
	heading.add_theme_color_override("font_color", Theme_.ACCENT)
	add_child(heading)

	tree.columns = 6
	tree.column_titles_visible = true
	tree.hide_root = true
	tree.select_mode = Tree.SELECT_ROW
	tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var titles := ["Commodity", "Buy", "Sell", "Available", "vs Base", "Hold"]
	for i in titles.size():
		tree.set_column_title(i, titles[i])
		tree.set_column_expand(i, true)
		tree.set_column_expand_ratio(i, 3 if i == 0 else 2)
		tree.set_column_custom_minimum_width(i, 110)
	add_child(tree)

	var controls := HBoxContainer.new()
	controls.add_theme_constant_override("separation", 10)
	quantity.min_value = 1
	quantity.max_value = 100
	quantity.value = 1
	quantity.prefix = "Tons"
	quantity.custom_minimum_size = Vector2(160, 0)
	controls.add_child(quantity)
	var buy := Button.new()
	buy.text = "Buy"
	buy.pressed.connect(_trade.bind(true))
	controls.add_child(buy)
	var sell := Button.new()
	sell.text = "Sell"
	sell.pressed.connect(_trade.bind(false))
	controls.add_child(sell)
	add_child(controls)
	add_child(feedback)
	refresh()


func refresh() -> void:
	var market: Dictionary = game.market()
	heading.text = "📊  %s Market" % market["system_name"]
	var selected_id := _selected_id()
	tree.clear()
	var root := tree.create_item()
	for row in market["rows"]:
		var item := tree.create_item(root)
		item.set_metadata(0, row["id"])
		item.set_text(0, row["name"])
		item.set_text(1, "%d cr" % row["buy_price"])
		item.set_text(2, "%d cr" % row["sell_price"])
		item.set_text(3, "%d t" % row["available"])
		item.set_text(4, "%+d%%" % row["percent_vs_base"])
		if row["held"] > 0:
			var paid: int = row["paid"]
			item.set_text(5, "%d t @ %d cr" % [row["held"], paid] if paid >= 0 else "%d t" % row["held"])
			item.set_custom_color(5, Theme_.ACCENT)
		for column in range(1, 6):
			item.set_text_alignment(column, HORIZONTAL_ALIGNMENT_RIGHT)
		if row["percent_vs_base"] <= -10:
			item.set_custom_color(4, Theme_.GOOD)
		elif row["percent_vs_base"] >= 10:
			item.set_custom_color(4, Theme_.BAD)
		if row["id"] == selected_id:
			item.select(0)


func _selected_id() -> String:
	var item := tree.get_selected()
	return item.get_metadata(0) if item else ""


func _trade(buying: bool) -> void:
	var id := _selected_id()
	if id == "":
		_say("Select a commodity first", false)
		return
	var amount := int(quantity.value)
	var result: Dictionary = game.buy(id, amount) if buying else game.sell(id, amount)
	_say(result["message"], result["ok"])
	refresh()
	traded.emit()


func _say(message: String, ok: bool) -> void:
	feedback.text = ("✅  " if ok else "❌  ") + message
	feedback.add_theme_color_override("font_color", Theme_.GOOD if ok else Theme_.BAD)
