extends Node3D
## The space between the jump and the station: a wireframe ship and a starfield that streaks during a jump.

const Style := preload("res://theme/visual_style.gd")

const PLAYER_MODEL := "cobra-mk-iii"
const ENEMY_MODEL := "raider"
const STAR_COUNT := 260
const FIELD := Vector3(60, 36, 120)
const CRUISE_SPEED := 4.0
const JUMP_SPEED := 160.0
const JUMP_SECONDS := 3.0

const PLAYER_ENCOUNTER := Vector3(-3.4, -0.7, 0.0)
const ENEMY_ENCOUNTER := Vector3(4.4, 0.9, -15.0)

var ship: MeshInstance3D
var enemy: MeshInstance3D
var world := WorldEnvironment.new()
var in_encounter := false
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

	world.environment = Style.environment()
	add_child(world)

	ship = Style.model_node(PLAYER_MODEL, "player")
	add_child(ship)

	enemy = Style.model_node(ENEMY_MODEL, "enemy")
	enemy.visible = false
	add_child(enemy)

	stars.mesh = star_mesh
	stars.material_override = Style.material("stars")
	add_child(stars)
	for i in STAR_COUNT:
		star_positions.append(_random_star(true))
	Style.events.changed.connect(_restyle)


func _restyle() -> void:
	world.environment = Style.environment()
	Style.restyle(ship, PLAYER_MODEL, "player")
	Style.restyle(enemy, ENEMY_MODEL, "enemy")
	stars.material_override = Style.material("stars")


func start_jump() -> void:
	jump_left = JUMP_SECONDS


func begin_encounter() -> void:
	in_encounter = true
	ship.position = PLAYER_ENCOUNTER
	enemy.position = ENEMY_ENCOUNTER
	enemy.visible = true
	enemy.scale = Vector3.ONE


func end_encounter() -> void:
	in_encounter = false
	ship.position = Vector3.ZERO
	ship.visible = true
	enemy.visible = false


func point_of(player: bool) -> Vector3:
	return ship.position if player else enemy.position


## A burst of laser fire from one ship at the other; a miss passes wide.
func fire_beam(from_player: bool, hit: bool) -> void:
	var from := point_of(from_player)
	var target := point_of(not from_player)
	if not hit:
		target += Vector3(randf_range(-4.0, 4.0), randf_range(2.0, 4.0), 0.0)
	var material := Style.material("beam_player" if from_player else "beam_enemy", true)
	var mesh := ImmediateMesh.new()
	mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	for offset in [Vector3.ZERO, Vector3(0.05, 0.05, 0), Vector3(-0.05, 0.05, 0), Vector3(0, -0.06, 0)]:
		mesh.surface_add_vertex(from + offset)
		mesh.surface_add_vertex(target + offset)
	mesh.surface_end()
	var beam := MeshInstance3D.new()
	beam.mesh = mesh
	beam.material_override = material
	add_child(beam)
	_fade_and_free(beam, material, 0.4, 1.0)


## A shimmering wireframe shield around the ship that was hit.
func flash_shield(on_player: bool) -> void:
	var material := Style.material("shield", true)
	var bubble := MeshInstance3D.new()
	bubble.mesh = _sphere_mesh(2.6)
	bubble.material_override = material
	bubble.position = point_of(on_player)
	add_child(bubble)
	_fade_and_free(bubble, material, 0.5, 1.25)


func explode(on_player: bool) -> void:
	var at := point_of(on_player)
	var material := Style.material("explosion", true)
	var burst := MeshInstance3D.new()
	var mesh := ImmediateMesh.new()
	mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	for i in 70:
		var direction := Vector3(randf_range(-1, 1), randf_range(-1, 1), randf_range(-1, 1)).normalized()
		mesh.surface_add_vertex(direction * randf_range(0.2, 0.8))
		mesh.surface_add_vertex(direction * randf_range(1.0, 2.2))
	mesh.surface_end()
	burst.mesh = mesh
	burst.material_override = material
	burst.position = at
	add_child(burst)
	_fade_and_free(burst, material, 1.1, 5.0)
	if on_player:
		ship.visible = false
	else:
		enemy.visible = false


func _fade_and_free(node: Node3D, material: StandardMaterial3D, seconds: float, grow: float) -> void:
	var tween := create_tween()
	tween.tween_property(material, "albedo_color:a", 0.0, seconds)
	tween.parallel().tween_property(node, "scale", Vector3.ONE * grow, seconds)
	tween.tween_callback(node.queue_free)


func _sphere_mesh(radius: float) -> ImmediateMesh:
	var mesh := ImmediateMesh.new()
	mesh.surface_begin(Mesh.PRIMITIVE_LINES)
	var rings := 6
	var steps := 24
	for ring in range(1, rings):
		var latitude := PI * ring / rings - PI / 2.0
		for i in steps:
			var a := TAU * i / steps
			var b := TAU * (i + 1) / steps
			mesh.surface_add_vertex(Vector3(cos(latitude) * cos(a), sin(latitude), cos(latitude) * sin(a)) * radius)
			mesh.surface_add_vertex(Vector3(cos(latitude) * cos(b), sin(latitude), cos(latitude) * sin(b)) * radius)
	for meridian in 8:
		var longitude := TAU * meridian / 8.0
		for i in steps:
			var a := TAU * i / steps
			var b := TAU * (i + 1) / steps
			mesh.surface_add_vertex(Vector3(cos(a) * cos(longitude), sin(a), cos(a) * sin(longitude)) * radius)
			mesh.surface_add_vertex(Vector3(cos(b) * cos(longitude), sin(b), cos(b) * sin(longitude)) * radius)
	mesh.surface_end()
	return mesh


func _process(delta: float) -> void:
	if enemy.visible:
		enemy.rotation.y = sin(Time.get_ticks_msec() / 1400.0) * 0.5 + 0.35
		enemy.position.y = ENEMY_ENCOUNTER.y + sin(Time.get_ticks_msec() / 700.0) * 0.15
	ship.rotation.y += delta * (0.4 if jump_left <= 0.0 and not in_encounter else 0.05)
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
