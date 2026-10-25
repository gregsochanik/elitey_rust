extends Control
## The first thing you see: start a new game or carry on from a save, including saves made by the CLI.

signal game_started

const Theme_ := preload("res://theme/elite_theme.gd")
const SavesList := preload("res://widgets/saves_list.gd")

var game: EliteGame
var name_edit := LineEdit.new()
var saves := SavesList.new()
var load_button := Button.new()
var delete_button := Button.new()
var message := Label.new()
var replace_dialog := ConfirmationDialog.new()
var delete_dialog := ConfirmationDialog.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	var backdrop := ColorRect.new()
	backdrop.color = Theme_.BACKGROUND
	backdrop.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(backdrop)

	var centre := CenterContainer.new()
	centre.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(centre)

	var box := VBoxContainer.new()
	box.custom_minimum_size = Vector2(1100, 0)
	box.add_theme_constant_override("separation", 14)
	centre.add_child(box)

	var title := Label.new()
	title.text = "🚀  ELITE"
	title.add_theme_font_size_override("font_size", 56)
	title.add_theme_color_override("font_color", Theme_.ACCENT)
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(title)

	var new_row := HBoxContainer.new()
	new_row.add_theme_constant_override("separation", 10)
	name_edit.placeholder_text = "Commander name"
	name_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	name_edit.text_submitted.connect(func(_text): _new_game())
	new_row.add_child(name_edit)
	var new_button := Button.new()
	new_button.text = "New game"
	new_button.pressed.connect(_new_game)
	new_row.add_child(new_button)
	box.add_child(new_row)

	var saved := Label.new()
	saved.text = "Continue a saved game"
	saved.add_theme_font_size_override("font_size", Theme_.FONT_STRONG)
	box.add_child(saved)

	saves.setup(game)
	saves.custom_minimum_size = Vector2(0, 260)
	saves.item_selected.connect(_update_buttons)
	box.add_child(saves)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	load_button.text = "Load selected"
	load_button.pressed.connect(_load)
	row.add_child(load_button)
	delete_button.text = "Delete selected"
	delete_button.pressed.connect(func(): delete_dialog.popup_centered())
	row.add_child(delete_button)
	box.add_child(row)
	box.add_child(message)

	var credit := Label.new()
	credit.text = "Star systems from the HYG database (CC BY-SA 4.0)"
	credit.add_theme_font_size_override("font_size", Theme_.FONT_SMALL)
	credit.add_theme_color_override("font_color", Theme_.MUTED)
	credit.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(credit)

	replace_dialog.title = "Replace save?"
	replace_dialog.confirmed.connect(_start_new)
	add_child(replace_dialog)
	delete_dialog.title = "Delete save"
	delete_dialog.dialog_text = "Delete the selected save? This cannot be undone."
	delete_dialog.confirmed.connect(_delete)
	add_child(delete_dialog)
	_update_buttons()
	name_edit.grab_focus()


func _update_buttons() -> void:
	var has := saves.selected_slot() != ""
	load_button.disabled = not has
	delete_button.disabled = not has


func _new_game() -> void:
	var name := name_edit.text.strip_edges()
	if name == "":
		_say("Enter a commander name first", false)
		return
	if game.autosave_exists(name):
		replace_dialog.dialog_text = "%s already has a saved game. Start over and replace it when you quit?" % name
		replace_dialog.popup_centered()
		return
	_start_new()


func _start_new() -> void:
	var result: Dictionary = game.new_game(name_edit.text)
	if result["ok"]:
		game_started.emit()
	else:
		_say(result["message"], false)


func _load() -> void:
	var result: Dictionary = game.load_slot(saves.selected_slot())
	if result["ok"]:
		game_started.emit()
	else:
		_say(result["message"], false)


func _delete() -> void:
	var result: Dictionary = game.delete_slot(saves.selected_slot())
	_say(result["message"], result["ok"])
	saves.refresh()
	_update_buttons()


func _say(text: String, ok: bool) -> void:
	message.text = ("✅  " if ok else "❌  ") + text
	message.add_theme_color_override("font_color", Theme_.GOOD if ok else Theme_.BAD)
