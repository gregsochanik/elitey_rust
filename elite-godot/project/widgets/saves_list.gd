extends Tree
## The saves on disk, newest first. Shared by the start screen and the Saves screen.

const Theme_ := preload("res://theme/elite_theme.gd")

var game: EliteGame


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	columns = 7
	column_titles_visible = true
	hide_root = true
	select_mode = Tree.SELECT_ROW
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	var titles := ["Slot", "Commander", "Location", "Credits", "Ship", "Saved", ""]
	for i in titles.size():
		set_column_title(i, titles[i])
		set_column_expand(i, true)
		set_column_expand_ratio(i, 3 if i in [0, 5] else 2)
	refresh()


func refresh() -> void:
	var keep := selected_slot()
	clear()
	var root := create_item()
	var first: TreeItem = null
	for save in game.saves():
		var item := create_item(root)
		item.set_metadata(0, save["slot"])
		item.set_text(0, save["slot"])
		item.set_text(1, save["commander"])
		item.set_text(2, save["location"])
		item.set_text(3, "%d cr" % save["credits"])
		item.set_text(4, save["ship"])
		item.set_text(5, save["saved_at"])
		item.set_text_alignment(3, HORIZONTAL_ALIGNMENT_RIGHT)
		if save["is_autosave"]:
			item.set_text(6, "autosave")
			item.set_custom_color(6, Theme_.MUTED)
		if first == null:
			first = item
		if save["slot"] == keep:
			item.select(0)
	if keep == "" and first != null:
		first.select(0)
	if first == null:
		var empty := create_item(root)
		empty.set_text(0, "No saves yet")
		empty.set_custom_color(0, Theme_.MUTED)


func selected_slot() -> String:
	var item := get_selected()
	if item == null or item.get_metadata(0) == null:
		return ""
	return item.get_metadata(0)


func selected_commander() -> String:
	var item := get_selected()
	return item.get_text(1) if item and item.get_metadata(0) != null else ""
