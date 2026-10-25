extends Control
## A jump in progress: hyperspace, then each event, inspection or fight the core hands over, then arrival.

signal arrived
signal started

const Theme_ := preload("res://theme/elite_theme.gd")
const JourneyScene := preload("res://screens/journey_scene.gd")
const DialogPanel := preload("res://widgets/dialog_panel.gd")
var hyperspace_seconds := 2.6

var game: EliteGame
var journey: Node3D
var log := RichTextLabel.new()
var dialog := DialogPanel.new()
var banner := Label.new()
var fight_panel := PanelContainer.new()
var fight_logged := 0
var fight_buttons: Array[Button] = []
var player_bars := {}
var enemy_bars := {}


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

	_build_fight_panel()

	add_child(dialog)

	banner.text = "🚀  Cruising"


func start(destination: String) -> void:
	log.clear()
	_clear_prompt()
	banner.text = "🚀  Jumping to %s" % destination
	started.emit()
	journey.start_jump()
	var step: Dictionary = game.begin_jump(destination)
	_log_notices(step)
	await get_tree().create_timer(hyperspace_seconds).timeout
	_present(step)


func _present(step: Dictionary) -> void:
	_clear_prompt()
	fight_panel.visible = false
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


## What came of an answer: shown as an outcome dialog before the journey moves on.
func _answered(step: Dictionary) -> void:
	_log_notices(step)
	_clear_prompt()
	var lines := []
	for notice in step["notices"]:
		if notice["kind"] != "blank" and notice["text"] != "":
			lines.append(notice)
	if lines.is_empty():
		_present(step)
		return
	dialog.subtitle("Outcome")
	dialog.title("What happened")
	for notice in lines:
		dialog.text(notice["text"], _notice_colour(notice["kind"]))
	dialog.choice("Continue", func(): _present(step))
	dialog.present()


func _notice_colour(kind: String) -> Color:
	match kind:
		"success":
			return Theme_.GOOD
		"error":
			return Theme_.BAD
		"info":
			return Color("6fb1ff")
		_:
			return Theme_.TEXT


func _show_event(step: Dictionary) -> void:
	dialog.subtitle("Hyperspace" if step["phase"] == "jump" else "Approaching the station")
	dialog.title(step["title"])
	dialog.text(step["description"])
	for i in step["choices"].size():
		dialog.choice(step["choices"][i], func(): _answered(game.answer_event(i)))
	dialog.present()


func _build_fight_panel() -> void:
	fight_panel.visible = false
	fight_panel.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
	fight_panel.offset_top = -330
	fight_panel.offset_bottom = -186
	fight_panel.offset_left = 12
	fight_panel.offset_right = -12
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 24)
	fight_panel.add_child(row)

	row.add_child(_fighter_box(player_bars, Theme_.ACCENT))

	var actions := VBoxContainer.new()
	actions.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	actions.alignment = BoxContainer.ALIGNMENT_CENTER
	actions.add_theme_constant_override("separation", 8)
	for entry in [["⚔️  Attack", "attack"], ["💨  Flee", "flee"], ["🏳️  Surrender", "surrender"]]:
		var button := Button.new()
		button.text = entry[0]
		button.pressed.connect(_fight_action.bind(entry[1]))
		actions.add_child(button)
		fight_buttons.append(button)
	row.add_child(actions)

	row.add_child(_fighter_box(enemy_bars, Theme_.BAD))
	add_child(fight_panel)


func _fighter_box(bars: Dictionary, colour: Color) -> Control:
	var box := VBoxContainer.new()
	box.custom_minimum_size = Vector2(340, 0)
	box.add_theme_constant_override("separation", 6)
	var name := Label.new()
	name.add_theme_font_size_override("font_size", Theme_.FONT_STRONG)
	name.add_theme_color_override("font_color", colour)
	box.add_child(name)
	bars["name"] = name
	for key in ["hull", "shield"]:
		var label := Label.new()
		box.add_child(label)
		var bar := ProgressBar.new()
		bar.show_percentage = false
		bar.custom_minimum_size = Vector2(0, 14)
		var fill := StyleBoxFlat.new()
		fill.bg_color = Theme_.GOOD if key == "hull" else Color("4aa3ff")
		fill.set_corner_radius_all(4)
		bar.add_theme_stylebox_override("fill", fill)
		box.add_child(bar)
		bars[key + "_label"] = label
		bars[key] = bar
	return box


func _show_fight(step: Dictionary) -> void:
	journey.begin_encounter()
	fight_logged = 0
	banner.text = "⚔️  %s attacks!" % step["enemy_name"]
	_update_fight(game.fight())
	fight_panel.visible = true
	_set_fight_buttons(true)


func _update_fight(fight: Dictionary) -> void:
	_update_bars(player_bars, fight["player"])
	_update_bars(enemy_bars, fight["enemy"])


func _update_bars(bars: Dictionary, fighter: Dictionary) -> void:
	bars["name"].text = fighter["name"]
	bars["hull_label"].text = "Hull  %d / %d" % [fighter["hull"], fighter["max_hull"]]
	bars["hull"].max_value = maxf(fighter["max_hull"], 1)
	bars["hull"].value = fighter["hull"]
	bars["shield_label"].text = "Shield  %d / %d" % [fighter["shield"], fighter["max_shield"]]
	bars["shield"].max_value = maxf(fighter["max_shield"], 1)
	bars["shield"].value = fighter["shield"]


func _set_fight_buttons(enabled: bool) -> void:
	for button in fight_buttons:
		button.disabled = not enabled


func _fight_action(action: String) -> void:
	_set_fight_buttons(false)
	var fight: Dictionary = game.fight_action(action)
	if fight["message"] != "":
		log.append_text("[color=#ef6f6c]❌ %s[/color]\n" % fight["message"])
		_set_fight_buttons(true)
		return
	var lines: Array = fight["log"].slice(fight_logged)
	fight_logged = fight["log"].size()
	var player_name: String = fight["player"]["name"]
	for line: String in lines:
		log.append_text("%s\n" % line)
		_animate_line(line, player_name)
		await get_tree().create_timer(0.55).timeout
	_update_fight(fight)
	if fight["outcome"] == "ongoing":
		_set_fight_buttons(true)
		return
	await get_tree().create_timer(0.8).timeout
	_end_fight(fight)


func _animate_line(line: String, player_name: String) -> void:
	if " hits " in line or " misses " in line:
		var from_player := line.begins_with(player_name)
		var hit := " hits " in line
		journey.fire_beam(from_player, hit)
		if hit:
			journey.flash_shield(not from_player)
	elif "breaks off and escapes" in line:
		journey.enemy_leaves()
	elif "surrenders!" in line and not line.begins_with("You"):
		journey.enemy_yields()
	elif "is destroyed" in line:
		journey.explode(false)
	elif "crippled" in line:
		journey.explode(true)


func _end_fight(fight: Dictionary) -> void:
	fight_panel.visible = false
	var enemy_name: String = fight["enemy"]["name"]
	var title := ""
	var text := ""
	match fight["outcome"]:
		"victory":
			title = "🏆  Victory"
			text = "The %s is destroyed." % enemy_name
		"defeat":
			title = "💥  Defeated"
			text = "Your ship is crippled."
		"escaped":
			title = "💨  Escaped"
			text = "You get away from the %s." % enemy_name
		"enemy_fled":
			title = "💨  It got away"
			text = "The %s breaks off and escapes." % enemy_name
		"enemy_surrendered":
			title = "🏳️  It surrenders"
			text = "The %s gives in. You take what it offers." % enemy_name
		_:
			title = "🏳️  Surrendered"
			text = "You give in to the %s." % enemy_name
	_clear_prompt()
	_show_message(title, text, "Continue", func():
		journey.end_encounter()
		_next(game.finish_fight()))


func _show_inspection(step: Dictionary) -> void:
	dialog.subtitle("Approaching the station")
	dialog.title("🚨  Patrol inspection")
	dialog.text("A patrol vessel has stopped you and is scanning your hold.")
	var bribe := dialog.choice("Bribe the patrol (%d cr)" % step["bribe_cost"], func(): _answered(game.answer_inspection("bribe")))
	bribe.disabled = not step["can_bribe"]
	dialog.choice("Run for it", func(): _answered(game.answer_inspection("run")))
	dialog.choice("Hand over the cargo", func(): _answered(game.answer_inspection("hand_over")))
	dialog.present()


func _show_arrival(step: Dictionary) -> void:
	banner.text = "🚀  Arrived at %s" % step["arrived_at"]
	var title := "Docked at %s" % step["arrived_at"]
	var text := ""
	if step["arrived_at"] != step["planned"]:
		text = "You were diverted from %s." % step["planned"]
	_show_message(title, text, "Continue", func(): arrived.emit())


func _show_message(title: String, text: String, button: String, action: Callable) -> void:
	dialog.title(title)
	if text != "":
		dialog.text(text)
	dialog.choice(button, action)
	dialog.present()


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


func _clear_prompt() -> void:
	dialog.clear()
