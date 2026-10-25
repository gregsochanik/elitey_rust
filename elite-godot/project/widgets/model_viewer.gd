extends VBoxContainer
## A turntable for one model: drag to turn it, scroll to zoom, pick the visual style.

const Style := preload("res://theme/visual_style.gd")
const Theme_ := preload("res://theme/elite_theme.gd")

const SPIN := 0.5

var model_id := ""
var role := "player"
var container := SubViewportContainer.new()
var viewport := SubViewport.new()
var camera := Camera3D.new()
var world := WorldEnvironment.new()
var node: MeshInstance3D
var lights := []
var style_picker := OptionButton.new()
var dragging := false
var yaw := 0.6
var pitch := -0.3
var zoom := 1.0


func _ready() -> void:
	add_theme_constant_override("separation", 6)
	container.stretch = true
	container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	container.custom_minimum_size = Vector2(300, 240)
	viewport.own_world_3d = true
	viewport.msaa_3d = Viewport.MSAA_4X
	container.add_child(viewport)
	camera.keep_aspect = Camera3D.KEEP_WIDTH
	viewport.add_child(camera)
	world.environment = Style.environment()
	viewport.add_child(world)
	add_child(container)

	for name in Style.names():
		style_picker.add_item(name)
	style_picker.select(Style.names().find(Style.current()))
	style_picker.item_selected.connect(func(index): Style.use(style_picker.get_item_text(index)))
	add_child(style_picker)

	_relight()
	container.gui_input.connect(_input_on_view)
	Style.events.changed.connect(_restyle)
	_rebuild()


func show_model(id: String, new_role := "player") -> void:
	model_id = id
	role = new_role
	if is_node_ready():
		_rebuild()


func _rebuild() -> void:
	if node:
		node.queue_free()
		node = null
	if model_id == "":
		return
	node = Style.model_node(model_id, role)
	viewport.add_child(node)
	_frame()


func _restyle() -> void:
	world.environment = Style.environment()
	_rebuild()
	_relight()


func _relight() -> void:
	for light in lights:
		light.queue_free()
	lights = Style.lights()
	for light in lights:
		viewport.add_child(light)


func _frame() -> void:
	var radius := node.mesh.get_aabb().size.length() * 0.5
	camera.position = Vector3(0, 0, maxf(radius, 1.0) * 1.8 * zoom)


func _process(delta: float) -> void:
	if node == null:
		return
	if not dragging:
		yaw += delta * SPIN
	node.rotation = Vector3(pitch, yaw, 0)


func _input_on_view(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT:
			dragging = event.pressed
		elif event.pressed and event.button_index == MOUSE_BUTTON_WHEEL_UP:
			zoom = maxf(zoom * 0.9, 0.4)
			_frame()
		elif event.pressed and event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			zoom = minf(zoom * 1.1, 2.5)
			_frame()
	elif event is InputEventMouseMotion and dragging:
		yaw += event.relative.x * 0.01
		pitch = clampf(pitch + event.relative.y * 0.01, -1.4, 1.4)
