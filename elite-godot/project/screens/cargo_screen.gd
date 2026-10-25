extends VBoxContainer
## What is in the hold and what it cost.

const Theme_ := preload("res://theme/elite_theme.gd")

var game: EliteGame
var heading := Label.new()
var contract := Label.new()
var tree := Tree.new()
var total := Label.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	add_theme_constant_override("separation", 10)
	heading.add_theme_font_size_override("font_size", 30)
	heading.add_theme_color_override("font_color", Theme_.ACCENT)
	add_child(heading)

	contract.add_theme_color_override("font_color", Theme_.WARNING)
	add_child(contract)

	tree.columns = 4
	tree.column_titles_visible = true
	tree.hide_root = true
	tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var titles := ["Commodity", "Tons", "Value", "Paid / ton"]
	for i in titles.size():
		tree.set_column_title(i, titles[i])
		tree.set_column_expand(i, true)
		tree.set_column_expand_ratio(i, 3 if i == 0 else 2)
	add_child(tree)

	total.add_theme_font_size_override("font_size", 24)
	add_child(total)


func refresh() -> void:
	var cargo: Dictionary = game.cargo()
	heading.text = "📦  Cargo Hold  (%d/%d t)" % [cargo["weight"], cargo["capacity"]]
	contract.text = "Contract cargo: %d t (not for sale)" % cargo["reserved"] if cargo["reserved"] > 0 else ""
	contract.visible = cargo["reserved"] > 0
	tree.clear()
	var root := tree.create_item()
	for line in cargo["items"]:
		var item := tree.create_item(root)
		item.set_text(0, line["name"])
		item.set_text(1, "%d" % line["quantity"])
		item.set_text(2, "%d cr" % line["estimated_value"])
		item.set_text(3, "%d cr" % line["paid"] if line["paid"] >= 0 else "—")
		for column in range(1, 4):
			item.set_text_alignment(column, HORIZONTAL_ALIGNMENT_RIGHT)
	if cargo["items"].is_empty():
		var empty := tree.create_item(root)
		empty.set_text(0, "No goods in the hold")
		empty.set_custom_color(0, Theme_.MUTED)
	total.text = "Total value: %d cr" % cargo["total_value"]
