extends VBoxContainer
## The board here and the jobs being worked on.

const Theme_ := preload("res://theme/elite_theme.gd")

var game: EliteGame
var heading := Label.new()
var board_tree := Tree.new()
var missions_tree := Tree.new()
var accept_button := Button.new()
var abandon_button := Button.new()
var feedback := Label.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	add_theme_constant_override("separation", 8)
	heading.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	heading.add_theme_color_override("font_color", Theme_.ACCENT)
	add_child(heading)

	_prepare(board_tree, ["#", "Job", "Distance", "Fuel", "Reward", "Deadline", ""])
	board_tree.item_selected.connect(_update_buttons)
	board_tree.size_flags_stretch_ratio = 3.0
	add_child(board_tree)

	accept_button.text = "Accept selected job"
	accept_button.pressed.connect(func(): _act(game.accept_job(_selected_number(board_tree))))
	add_child(accept_button)

	var title := Label.new()
	title.text = "Active jobs"
	title.add_theme_font_size_override("font_size", Theme_.FONT_SECTION)
	add_child(title)

	_prepare(missions_tree, ["#", "Job", "Reward", "Time left"])
	missions_tree.item_selected.connect(_update_buttons)
	missions_tree.size_flags_stretch_ratio = 2.0
	add_child(missions_tree)

	abandon_button.text = "Abandon selected job (−3 reputation)"
	abandon_button.pressed.connect(func(): _act(game.abandon_job(_selected_number(missions_tree))))
	add_child(abandon_button)
	add_child(feedback)
	refresh()


func _prepare(tree: Tree, titles: Array) -> void:
	tree.columns = titles.size()
	tree.column_titles_visible = true
	tree.hide_root = true
	tree.select_mode = Tree.SELECT_ROW
	tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	for i in titles.size():
		tree.set_column_title(i, titles[i])
		tree.set_column_expand(i, true)
		tree.set_column_expand_ratio(i, 1 if titles[i] == "#" else (6 if titles[i] == "Job" else 2))


func refresh() -> void:
	var board: Dictionary = game.jobs()
	heading.text = "💼  Jobs at %s    (%d/%d active)" % [board["location"], board["active"], board["max_active"]]
	var keep := _selected_number(board_tree)
	board_tree.clear()
	var root := board_tree.create_item()
	for job in board["jobs"]:
		var item := board_tree.create_item(root)
		item.set_metadata(0, job["number"])
		item.set_metadata(1, job)
		item.set_text(0, "%d" % job["number"])
		item.set_text(1, job["description"])
		item.set_text(2, "%.1f ly" % job["distance"])
		item.set_text(3, "%d" % job["fuel_cost"])
		item.set_text(4, "%d cr" % job["reward"])
		item.set_text(5, "%d jumps" % job["deadline_jumps"])
		for column in [0, 2, 3, 4, 5]:
			item.set_text_alignment(column, HORIZONTAL_ALIGNMENT_RIGHT)
		if job["locked"]:
			item.set_text(6, "🔒 " + job["lock_reason"])
			for column in range(0, 7):
				item.set_custom_color(column, Theme_.MUTED)
			item.set_custom_color(6, Theme_.BAD)
		else:
			item.set_custom_color(4, Theme_.GOOD)
		if job["number"] == keep:
			item.select(0)
	if board["jobs"].is_empty():
		var empty := board_tree.create_item(root)
		empty.set_text(1, "No jobs on offer here")

	missions_tree.clear()
	root = missions_tree.create_item()
	for mission in game.missions():
		var item := missions_tree.create_item(root)
		item.set_metadata(0, mission["number"])
		item.set_text(0, "%d" % mission["number"])
		item.set_text(1, mission["description"])
		item.set_text(2, "%d cr" % mission["reward"])
		item.set_text(3, "%d jumps" % mission["jumps_left"])
		item.set_custom_color(2, Theme_.GOOD)
		for column in [0, 2, 3]:
			item.set_text_alignment(column, HORIZONTAL_ALIGNMENT_RIGHT)
	_update_buttons()


func _selected_number(tree: Tree) -> int:
	var item := tree.get_selected()
	return item.get_metadata(0) if item else 0


func _update_buttons() -> void:
	var item := board_tree.get_selected()
	accept_button.disabled = item == null or item.get_metadata(1)["locked"]
	abandon_button.disabled = _selected_number(missions_tree) == 0


func _act(result: Dictionary) -> void:
	feedback.text = ("✅  " if result["ok"] else "❌  ") + result["message"]
	feedback.add_theme_color_override("font_color", Theme_.GOOD if result["ok"] else Theme_.BAD)
	refresh()
