extends Control
## A jump in progress: hyperspace, then each event, inspection or fight the core hands over, then arrival.

signal arrived
signal started

const Theme_ := preload("res://theme/elite_theme.gd")
const JourneyScene := preload("res://screens/journey_scene.gd")
const HYPERSPACE_SECONDS := 2.6

var game: EliteGame
var journey: Node3D
var log := RichTextLabel.new()
var prompt := PanelContainer.new()
var prompt_box := VBoxContainer.new()
var banner := Label.new()


func setup(the_game: EliteGame) -> void:
	game = the_game


func _ready() -> void:
	var container := SubViewportContainer.new()
	container.stretch = true
	container.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(container)
	var viewport := SubViewport.new()
	viewport.size = Vector2i(1000, 560)
	container.add_child(viewport)
	journey = JourneyScene.new()
	viewport.add_child(journey)

	banner.add_theme_font_size_override("font_size", Theme_.FONT_TITLE)
	banner.add_theme_color_override("font_color", Theme_.ACCENT)
	banner.set_anchors_and_offsets_preset(Control.PRESET_TOP_WIDE)
	banner.offset_top = 12
	banner.offset_left = 16
	add_child(banner)

	log.bbcode_enabled = true
	log.scroll_following = true
	log.fit_content = false
	log.custom_minimum_size = Vector2(0, 150)
	log.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
	log.offset_top = -170
	log.offset_left = 12
	log.offset_right = -12
	log.offset_bottom = -12
	add_child(log)

	prompt.visible = false
	prompt.custom_minimum_size = Vector2(560, 0)
	prompt.set_anchors_and_offsets_preset(Control.PRESET_CENTER)
	prompt.grow_horizontal = Control.GROW_DIRECTION_BOTH
	prompt.grow_vertical = Control.GROW_DIRECTION_BOTH
	prompt_box.add_theme_constant_override("separation", 12)
	prompt.add_child(prompt_box)
	add_child(prompt)

	banner.text = "🚀  Cruising"


func start(destination: String) -> void:
	log.clear()
	_clear_prompt()
	banner.text = "🚀  Jumping to %s" % destination
	started.emit()
	journey.start_jump()
	var step: Dictionary = game.begin_jump(destination)
	_log_notices(step)
	await get_tree().create_timer(HYPERSPACE_SECONDS).timeout
	_present(step)


func _present(step: Dictionary) -> void:
	_clear_prompt()
	match step["kind"]:
		"event":
			_show_event(step)
		"fight":
			_show_fight(step)
		"inspection":
			_show_inspection(step)
		"arrived":
			_show_arrival(step)
		_:
			_show_message("Journey problem", step["message"], "Continue", func(): arrived.emit())


func _next(step: Dictionary) -> void:
	_log_notices(step)
	_present(step)


func _show_event(step: Dictionary) -> void:
	_title(step["title"])
	_text(step["description"])
	for i in step["choices"].size():
		_button("%d.  %s" % [i + 1, step["choices"][i]], func(): _next(game.answer_event(i)))
	prompt.visible = true


func _show_fight(step: Dictionary) -> void:
	_title("⚔️  %s attacks!" % step["enemy_name"])
	_text("Your hull: %d    Enemy hull: %d" % [step["player_hull"], step["enemy_hull"]])
	_button("Fight", func(): _next(game.resolve_fight()))
	prompt.visible = true


func _show_inspection(step: Dictionary) -> void:
	_title("🚨  Patrol inspection")
	_text("A patrol vessel has stopped you and is scanning your hold.")
	var bribe := _button("Bribe (%d cr)" % step["bribe_cost"], func(): _next(game.answer_inspection("bribe")))
	bribe.disabled = not step["can_bribe"]
	_button("Run", func(): _next(game.answer_inspection("run")))
	_button("Hand over the cargo", func(): _next(game.answer_inspection("hand_over")))
	prompt.visible = true


func _show_arrival(step: Dictionary) -> void:
	banner.text = "🚀  Arrived at %s" % step["arrived_at"]
	var title := "Docked at %s" % step["arrived_at"]
	var text := ""
	if step["arrived_at"] != step["planned"]:
		text = "You were diverted from %s." % step["planned"]
	_show_message(title, text, "Continue", func(): arrived.emit())


func _show_message(title: String, text: String, button: String, action: Callable) -> void:
	_title(title)
	if text != "":
		_text(text)
	_button(button, action)
	prompt.visible = true


func _log_notices(step: Dictionary) -> void:
	for notice in step["notices"]:
		var text: String = notice["text"]
		match notice["kind"]:
			"blank":
				log.append_text("\n")
			"info":
				log.append_text("[color=#6fb1ff]ℹ️ %s[/color]\n" % text)
			"success":
				log.append_text("[color=#5ee08a]✅ %s[/color]\n" % text)
			"error":
				log.append_text("[color=#ef6f6c]❌ %s[/color]\n" % text)
			_:
				log.append_text("%s\n" % text)


func _title(text: String) -> void:
	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", Theme_.FONT_STRONG)
	label.add_theme_color_override("font_color", Theme_.ACCENT)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	prompt_box.add_child(label)


func _text(text: String) -> void:
	var label := Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	prompt_box.add_child(label)


func _button(text: String, action: Callable) -> Button:
	var button := Button.new()
	button.text = text
	button.pressed.connect(action)
	prompt_box.add_child(button)
	return button


func _clear_prompt() -> void:
	prompt.visible = false
	for child in prompt_box.get_children():
		prompt_box.remove_child(child)
		child.queue_free()
