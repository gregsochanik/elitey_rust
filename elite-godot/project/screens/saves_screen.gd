extends VBoxContainer
## Save, load and delete while playing.

signal loaded

const Theme_ := preload("res://theme/elite_theme.gd")
const SavesList := preload("res://widgets/saves_list.gd")

var game: EliteGame
var slot_edit := LineEdit.new()
var saves := SavesList.new()
var load_button := Button.new()
var delete_button := Button.new()
var feedback := Label.new()
var load_dialog := ConfirmationDialog.new()
var delete_dialog := ConfirmationDialog.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	add_theme_constant_override("separation", 10)
	var heading := Label.new()
	heading.text = "💾  Saves"
	heading.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	heading.add_theme_color_override("font_color", Theme_.ACCENT)
	add_child(heading)

	var save_row := HBoxContainer.new()
	save_row.add_theme_constant_override("separation", 10)
	slot_edit.placeholder_text = "Slot name (letters, numbers, underscores)"
	slot_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	slot_edit.text_submitted.connect(func(_text): _save())
	save_row.add_child(slot_edit)
	var save_button := Button.new()
	save_button.text = "Save game"
	save_button.pressed.connect(_save)
	save_row.add_child(save_button)
	add_child(save_row)

	saves.setup(game)
	saves.item_selected.connect(_update_buttons)
	add_child(saves)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	load_button.text = "Load selected"
	load_button.pressed.connect(func(): load_dialog.popup_centered())
	row.add_child(load_button)
	delete_button.text = "Delete selected"
	delete_button.pressed.connect(func(): delete_dialog.popup_centered())
	row.add_child(delete_button)
	add_child(row)
	add_child(feedback)

	var note := Label.new()
	note.text = "Your progress is also saved automatically when you quit."
	note.add_theme_color_override("font_color", Theme_.MUTED)
	add_child(note)

	load_dialog.title = "Load game"
	load_dialog.dialog_text = "Load the selected save? Unsaved progress will be lost."
	load_dialog.confirmed.connect(_load)
	add_child(load_dialog)
	delete_dialog.title = "Delete save"
	delete_dialog.dialog_text = "Delete the selected save? This cannot be undone."
	delete_dialog.confirmed.connect(_delete)
	add_child(delete_dialog)
	_update_buttons()


func refresh() -> void:
	saves.refresh()
	_update_buttons()


func _update_buttons() -> void:
	var has := saves.selected_slot() != ""
	load_button.disabled = not has
	delete_button.disabled = not has


func _save() -> void:
	var result: Dictionary = game.save_slot(slot_edit.text.strip_edges())
	_say(result)
	if result["ok"]:
		slot_edit.text = ""
	refresh()


func _load() -> void:
	var result: Dictionary = game.load_slot(saves.selected_slot())
	_say(result)
	if result["ok"]:
		loaded.emit()


func _delete() -> void:
	_say(game.delete_slot(saves.selected_slot()))
	refresh()


func _say(result: Dictionary) -> void:
	feedback.text = ("✅  " if result["ok"] else "❌  ") + result["message"]
	feedback.add_theme_color_override("font_color", Theme_.GOOD if result["ok"] else Theme_.BAD)
