extends Node3D
## The space between the jump and the station: a wireframe ship and a starfield that streaks during a jump.

const STAR_COUNT := 260
const FIELD := Vector3(60, 36, 120)
const CRUISE_SPEED := 4.0
const JUMP_SPEED := 160.0
const JUMP_SECONDS := 3.0

var ship := MeshInstance3D.new()
var stars := MeshInstance3D.new()
var star_mesh := ImmediateMesh.new()
var star_positions: PackedVector3Array = []
var speed := CRUISE_SPEED
var jump_left := 0.0


func _ready() -> void:
	var camera := Camera3D.new()
	camera.position = Vector3(0, 1.6, 9)
	camera.rotation_degrees = Vector3(-8, 0, 0)
	add_child(camera)

	var environment := Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = Color.BLACK
	environment.glow_enabled = true
	environment.glow_intensity = 1.2
	environment.glow_bloom = 0.15
	var world := WorldEnvironment.new()
	world.environment = environment
	add_child(world)

	ship.mesh = _ship_mesh()
	ship.material_override = _line_material(Color(0.35, 1.0, 0.85))
	add_child(ship)

	stars.mesh = star_mesh
	stars.material_override = _line_material(Color(0.8, 0.9, 1.0))
	add_child(stars)
	for i in STAR_COUNT:
		star_positions.append(_random_star(true))


func start_jump() -> void:
	jump_left = JUMP_SECONDS


func _process(delta: float) -> void:
	ship.rotation.y += delta * (0.4 if jump_left <= 0.0 else 0.05)
	ship.rotation.z = sin(Time.get_ticks_msec() / 900.0) * 0.12

	var target := JUMP_SPEED if jump_left > 0.0 else CRUISE_SPEED
	jump_left = maxf(jump_left - delta, 0.0)
	speed = lerpf(speed, target, minf(delta * 3.0, 1.0))

	star_mesh.clear_surfaces()
	star_mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	for i in star_positions.size():
		var star := star_positions[i]
		var tail := star - Vector3(0, 0, maxf(speed * 0.03, 0.04))
		star_mesh.surface_add_vertex(star)
		star_mesh.surface_add_vertex(tail)
		star.z += speed * delta
		if star.z > 12.0:
			star = _random_star(false)
		star_positions[i] = star
	star_mesh.surface_end()


func _random_star(anywhere: bool) -> Vector3:
	var z := randf_range(-FIELD.z, 12.0) if anywhere else -FIELD.z
	var x := randf_range(-FIELD.x, FIELD.x)
	var y := randf_range(-FIELD.y, FIELD.y)
	if absf(x) < 3.0 and absf(y) < 2.0:
		x += 6.0
	return Vector3(x, y, z)


func _line_material(color: Color) -> StandardMaterial3D:
	var material := StandardMaterial3D.new()
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.albedo_color = color
	return material


func _ship_mesh() -> ImmediateMesh:
	var v := [
		Vector3(0, 0, -3.2),
		Vector3(-0.9, 0.35, 0.6), Vector3(0.9, 0.35, 0.6),
		Vector3(-0.9, -0.2, 0.6), Vector3(0.9, -0.2, 0.6),
		Vector3(-3.0, 0.0, 1.8), Vector3(3.0, 0.0, 1.8),
		Vector3(0, 0.7, 1.8), Vector3(0, -0.4, 1.8),
	]
	var edges := [
		[0, 1], [0, 2], [0, 3], [0, 4],
		[1, 2], [3, 4], [1, 3], [2, 4],
		[1, 5], [3, 5], [2, 6], [4, 6],
		[1, 7], [2, 7], [3, 8], [4, 8],
		[5, 7], [6, 7], [5, 8], [6, 8],
		[5, 6],
	]
	var mesh := ImmediateMesh.new()
	mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	for edge in edges:
		mesh.surface_add_vertex(v[edge[0]])
		mesh.surface_add_vertex(v[edge[1]])
	mesh.surface_end()
	return mesh
