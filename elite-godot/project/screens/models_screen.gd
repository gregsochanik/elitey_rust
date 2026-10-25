extends HBoxContainer
## Developer view: every model file in turn, in any style.

const ModelLoader := preload("res://models/model_loader.gd")
const ModelViewer := preload("res://widgets/model_viewer.gd")
const Theme_ := preload("res://theme/elite_theme.gd")

const ENEMIES := ["raider"]

var list := ItemList.new()
var viewer := ModelViewer.new()
var details := Label.new()


func setup(_game: EliteGame) -> void:
	pass


func _ready() -> void:
	add_theme_constant_override("separation", 12)
	list.custom_minimum_size = Vector2(220, 0)
	for id in ModelLoader.ids():
		list.add_item(id)
	list.item_selected.connect(_show)
	add_child(list)

	var right := VBoxContainer.new()
	right.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	viewer.size_flags_vertical = Control.SIZE_EXPAND_FILL
	right.add_child(viewer)
	right.add_child(details)
	add_child(right)
	if list.item_count > 0:
		list.select(0)
		_show(0)


func _unhandled_key_input(event: InputEvent) -> void:
	if not is_visible_in_tree() or list.item_count == 0 or not event.pressed:
		return
	var step := 0
	if event.keycode == KEY_DOWN or event.keycode == KEY_RIGHT:
		step = 1
	elif event.keycode == KEY_UP or event.keycode == KEY_LEFT:
		step = -1
	if step != 0:
		var next := posmod(list.get_selected_items()[0] + step, list.item_count)
		list.select(next)
		_show(next)


func _show(index: int) -> void:
	var id := list.get_item_text(index)
	viewer.show_model(id, "enemy" if id in ENEMIES else "player")
	var model := ModelLoader.load_model(id)
	details.text = "%s · %d vertices · %d edges" % [id, model["vertices"].size(), model["edges"].size() / 2]
