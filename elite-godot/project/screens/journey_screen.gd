extends VBoxContainer
## The space between the jump and the station.

const JourneyScene := preload("res://screens/journey_scene.gd")

var journey: Node3D


func _ready() -> void:
	var container := SubViewportContainer.new()
	container.stretch = true
	container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_child(container)

	var viewport := SubViewport.new()
	viewport.size = Vector2i(1000, 560)
	container.add_child(viewport)
	journey = JourneyScene.new()
	viewport.add_child(journey)

	var jump := Button.new()
	jump.text = "🚀  Jump"
	jump.pressed.connect(func(): journey.start_jump())
	add_child(jump)
