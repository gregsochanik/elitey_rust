extends VBoxContainer
## Recent trades and totals.

const Theme_ := preload("res://theme/elite_theme.gd")
const RECENT := 25

var game: EliteGame
var heading := Label.new()
var tree := Tree.new()
var stats := Label.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	add_theme_constant_override("separation", 10)
	heading.text = "📜  Trading Log"
	heading.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	heading.add_theme_color_override("font_color", Theme_.ACCENT)
	add_child(heading)

	tree.columns = 6
	tree.column_titles_visible = true
	tree.hide_root = true
	tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var titles := ["Trade", "Qty", "Commodity", "Location", "Total", "Profit"]
	for i in titles.size():
		tree.set_column_title(i, titles[i])
		tree.set_column_expand(i, true)
		tree.set_column_expand_ratio(i, 2)
	add_child(tree)

	stats.add_theme_font_size_override("font_size", Theme_.FONT_BODY)
	add_child(stats)


func refresh() -> void:
	var log: Dictionary = game.trade_log(RECENT)
	tree.clear()
	var root := tree.create_item()
	for record in log["recent"]:
		var item := tree.create_item(root)
		item.set_text(0, "Bought" if record["bought"] else "Sold")
		item.set_text(1, "%d" % record["quantity"])
		item.set_text(2, record["commodity"])
		item.set_text(3, record["location"])
		item.set_text(4, "%d cr" % record["total"])
		if record["has_profit"]:
			item.set_text(5, "%+d cr" % record["profit"])
			item.set_custom_color(5, Theme_.GOOD if record["profit"] >= 0 else Theme_.BAD)
		for column in [1, 4, 5]:
			item.set_text_alignment(column, HORIZONTAL_ALIGNMENT_RIGHT)
	if log["recent"].is_empty():
		var empty := tree.create_item(root)
		empty.set_text(0, "No trades yet")
		empty.set_custom_color(0, Theme_.MUTED)

	var lines := ["Trades: %d" % log["total_trades"], "Net profit: %d cr" % log["net_profit"]]
	if log["has_success_rate"]:
		lines.append("Success rate: %.0f%%" % (log["success_rate"] * 100.0))
	if log["has_best"]:
		var best: Dictionary = log["best"]
		lines.append("Best trade: %+d cr (%d %s at %s)" % [best["profit"], best["quantity"], best["commodity"], best["location"]])
	stats.text = "   ·   ".join(lines)
