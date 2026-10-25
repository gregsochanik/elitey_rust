extends Control

const JourneyScene := preload("res://journey.gd")

var game := EliteGame.new()
var tree: Tree
var header: Label
var feedback: Label
var quantity: SpinBox
var journey: Node3D


func _ready() -> void:
	var tabs := TabContainer.new()
	tabs.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(tabs)

	var market_tab := _build_market()
	market_tab.name = "Market"
	tabs.add_child(market_tab)

	var journey_tab := _build_journey()
	journey_tab.name = "Journey"
	tabs.add_child(journey_tab)

	_refresh()


func _build_market() -> Control:
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 10)

	header = Label.new()
	header.add_theme_font_size_override("font_size", 22)
	box.add_child(header)

	tree = Tree.new()
	tree.columns = 6
	tree.column_titles_visible = true
	tree.hide_root = true
	tree.select_mode = Tree.SELECT_ROW
	tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var titles := ["Commodity", "Buy", "Sell", "Available", "vs Base", "Hold"]
	for i in titles.size():
		tree.set_column_title(i, titles[i])
		tree.set_column_expand(i, i == 0)
		tree.set_column_custom_minimum_width(i, 110)
	box.add_child(tree)

	var controls := HBoxContainer.new()
	quantity = SpinBox.new()
	quantity.min_value = 1
	quantity.max_value = 100
	quantity.value = 1
	quantity.prefix = "Tons"
	controls.add_child(quantity)
	var buy := Button.new()
	buy.text = "Buy"
	buy.pressed.connect(_trade.bind(true))
	controls.add_child(buy)
	var sell := Button.new()
	sell.text = "Sell"
	sell.pressed.connect(_trade.bind(false))
	controls.add_child(sell)
	box.add_child(controls)

	feedback = Label.new()
	box.add_child(feedback)
	return box


func _build_journey() -> Control:
	var box := VBoxContainer.new()
	var container := SubViewportContainer.new()
	container.stretch = true
	container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	box.add_child(container)

	var viewport := SubViewport.new()
	viewport.size = Vector2i(1000, 560)
	container.add_child(viewport)
	journey = JourneyScene.new()
	viewport.add_child(journey)

	var jump := Button.new()
	jump.text = "Jump"
	jump.pressed.connect(func(): journey.start_jump())
	box.add_child(jump)
	return box


func _refresh() -> void:
	var market: Dictionary = game.market()
	header.text = "%s Market    %d cr    %d/%d t" % [
		market["system_name"], market["credits"], market["cargo_used"], market["cargo_capacity"]
	]
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
		for column in range(1, 6):
			item.set_text_alignment(column, HORIZONTAL_ALIGNMENT_RIGHT)
		if row["percent_vs_base"] <= -10:
			item.set_custom_color(4, Color.GREEN)
		elif row["percent_vs_base"] >= 10:
			item.set_custom_color(4, Color.INDIAN_RED)
		if row["id"] == selected_id:
			item.select(0)


func _selected_id() -> String:
	var item := tree.get_selected() if tree else null
	return item.get_metadata(0) if item else ""


func _trade(buying: bool) -> void:
	var id := _selected_id()
	if id == "":
		feedback.text = "Select a commodity first"
		return
	var amount := int(quantity.value)
	var result: Dictionary = game.buy(id, amount) if buying else game.sell(id, amount)
	feedback.text = result["message"]
	feedback.add_theme_color_override("font_color", Color.GREEN if result["ok"] else Color.INDIAN_RED)
	_refresh()
