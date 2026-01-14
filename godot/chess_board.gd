extends Control

# Debug utilities
const DebugUtils = preload("res://debug_utils.gd")

# Board visual properties
const BOARD_SIZE = 640
const SQUARE_SIZE = BOARD_SIZE / 8
const LIGHT_SQUARE_COLOR = Color(0.93, 0.85, 0.71)
const DARK_SQUARE_COLOR = Color(0.72, 0.53, 0.38)
const SELECTED_COLOR = Color(0.8, 0.8, 0.3, 0.5)
const LEGAL_MOVE_COLOR = Color(0.3, 0.8, 0.3, 0.5)

# Piece colors
const PEARL_WHITE_COLOR = Color(0.97, 0.96, 0.94)
const BLACK_COLOR = Color(0.0, 0.0, 0.0)

# Game state
var chess_game: ChessGame
var selected_square: Vector2i = Vector2i(-1, -1)
var legal_moves: Array = []
var ai_plays_black: bool = false
var pending_promotion_move: Vector2i = Vector2i(-1, -1)

# Network state
var network_manager: Node
var is_online_mode: bool = false
var my_color: String = ""

# Piece tracking by ID (for persistent pieces and animations)
# Format: {piece_id: {position: Vector2i(row, col), label: Label}}
var pieces_by_id: Dictionary = {}

# Drag state
var is_dragging: bool = false
var drag_start_square: Vector2i = Vector2i(-1, -1)
var drag_mouse_pos: Vector2 = Vector2.ZERO

# UI nodes
var board_container: Control
var piece_labels: Array = []
var status_label: Label
var promotion_panel: Panel
var promotion_buttons: Dictionary = {}
var white_clock_label: Label
var black_clock_label: Label
var clock_timer: Timer
var ai_toggle: CheckBox
var clock_preset_dropdown: OptionButton

# Network UI nodes
var server_url_input: LineEdit
var player_id_input: LineEdit
var connect_button: Button
var network_status_label: Label
var game_info_label: Label

func _ready():
	DebugUtils.debug("ChessBoard _ready() called")

	# Enable mouse input for this Control node
	mouse_filter = Control.MOUSE_FILTER_STOP
	DebugUtils.debug("Mouse filter set to STOP")

	# Create the chess game instance
	chess_game = ChessGame.new()
	add_child(chess_game)
	DebugUtils.debug("ChessGame instance created and added as child")

	# Create network manager
	var NetworkManagerScript = preload("res://network_manager.gd")
	network_manager = NetworkManagerScript.new()
	add_child(network_manager)
	_connect_network_signals()
	DebugUtils.debug("NetworkManager created and added as child")

	# Setup UI
	setup_board()
	setup_status_label()
	setup_promotion_panel()
	setup_clock_display()
	setup_clock_preset_dropdown()
	setup_ai_toggle()
	setup_network_ui()

	# Start with default preset (3 | 2)
	apply_clock_preset(1)

	setup_clock_timer()

	update_board()
	update_clock_display()
	DebugUtils.debug("Board setup complete")

func setup_board():
	board_container = Control.new()
	board_container.position = Vector2(20, 60)
	board_container.custom_minimum_size = Vector2(BOARD_SIZE, BOARD_SIZE)
	# Allow mouse events to pass through to parent
	board_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(board_container)

	# Create labels for pieces
	# Flip board so white is at bottom (row 0 at bottom visually)
	for row in range(8):
		var row_array = []
		for col in range(8):
			var label = Label.new()
			var display_row = 7 - row  # Flip vertically so white is at bottom
			label.position = Vector2(col * SQUARE_SIZE, display_row * SQUARE_SIZE)
			label.custom_minimum_size = Vector2(SQUARE_SIZE, SQUARE_SIZE)
			label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
			label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER

			# Set font size
			label.add_theme_font_size_override("font_size", 48)

			# CRITICAL: Allow mouse events to pass through labels to parent Control
			label.mouse_filter = Control.MOUSE_FILTER_IGNORE

			board_container.add_child(label)
			row_array.append(label)
		piece_labels.append(row_array)

func setup_status_label():
	status_label = Label.new()
	status_label.position = Vector2(20, 20)
	status_label.custom_minimum_size = Vector2(BOARD_SIZE, 30)
	status_label.add_theme_font_size_override("font_size", 24)
	add_child(status_label)
	update_status()

func setup_promotion_panel():
	# Create a panel for promotion selection
	promotion_panel = Panel.new()
	promotion_panel.position = Vector2(BOARD_SIZE / 2 - 150, BOARD_SIZE / 2 + 30)
	promotion_panel.custom_minimum_size = Vector2(300, 150)
	promotion_panel.visible = false
	add_child(promotion_panel)

	# Title label
	var title_label = Label.new()
	title_label.text = "Promote pawn to:"
	title_label.position = Vector2(10, 10)
	title_label.add_theme_font_size_override("font_size", 20)
	promotion_panel.add_child(title_label)

	# Create buttons for each piece type
	var piece_types = ["Queen", "Rook", "Bishop", "Knight"]
	var button_width = 130
	var button_height = 40
	var start_y = 50

	for i in range(piece_types.size()):
		var piece_type = piece_types[i]
		var button = Button.new()
		button.text = piece_type
		button.custom_minimum_size = Vector2(button_width, button_height)
		var col = i % 2
		var row = i / 2
		button.position = Vector2(10 + col * (button_width + 10), start_y + row * (button_height + 10))
		button.pressed.connect(_on_promotion_selected.bind(piece_type.to_lower()))
		promotion_panel.add_child(button)
		promotion_buttons[piece_type.to_lower()] = button
func setup_clock_display():
	# White clock (bottom right - where white pieces start after flip)
	white_clock_label = Label.new()
	white_clock_label.position = Vector2(BOARD_SIZE + 40, 60 + BOARD_SIZE - 60)
	white_clock_label.custom_minimum_size = Vector2(150, 50)
	white_clock_label.add_theme_font_size_override("font_size", 28)
	white_clock_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	white_clock_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	add_child(white_clock_label)

	# Black clock (top right - where black pieces start after flip)
	black_clock_label = Label.new()
	black_clock_label.position = Vector2(BOARD_SIZE + 40, 60)
	black_clock_label.custom_minimum_size = Vector2(150, 50)
	black_clock_label.add_theme_font_size_override("font_size", 28)
	black_clock_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	black_clock_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	add_child(black_clock_label)

func setup_clock_timer():
	clock_timer = Timer.new()
	clock_timer.wait_time = 1.0  # Tick every second
	clock_timer.timeout.connect(_on_clock_tick)
	add_child(clock_timer)
	if chess_game.has_clock():
		clock_timer.start()

func setup_clock_preset_dropdown():
	# Clock preset dropdown
	clock_preset_dropdown = OptionButton.new()
	clock_preset_dropdown.position = Vector2(BOARD_SIZE + 40, 260)
	clock_preset_dropdown.custom_minimum_size = Vector2(150, 30)
	clock_preset_dropdown.add_theme_font_size_override("font_size", 14)

	# Add preset options
	clock_preset_dropdown.add_item("1 min", 0)
	clock_preset_dropdown.add_item("3 | 2", 1)
	clock_preset_dropdown.add_item("5 min", 2)
	clock_preset_dropdown.add_item("10 min", 3)
	clock_preset_dropdown.add_item("15 | 10", 4)
	clock_preset_dropdown.add_item("30 min", 5)

	# Set default selection
	clock_preset_dropdown.selected = 1  # "3 | 2"

	clock_preset_dropdown.item_selected.connect(_on_clock_preset_changed)
	add_child(clock_preset_dropdown)

func setup_ai_toggle():
	# AI toggle checkbox
	ai_toggle = CheckBox.new()
	ai_toggle.position = Vector2(BOARD_SIZE + 40, 200)
	ai_toggle.custom_minimum_size = Vector2(150, 30)
	ai_toggle.text = "AI plays Black"
	ai_toggle.add_theme_font_size_override("font_size", 16)
	ai_toggle.toggled.connect(_on_ai_toggle_changed)
	add_child(ai_toggle)

func setup_network_ui():
	# Network section label
	var network_label = Label.new()
	network_label.text = "Online Multiplayer"
	network_label.position = Vector2(BOARD_SIZE + 40, 160)
	network_label.add_theme_font_size_override("font_size", 18)
	add_child(network_label)

	# Server URL input
	server_url_input = LineEdit.new()
	server_url_input.position = Vector2(BOARD_SIZE + 40, 350)
	server_url_input.custom_minimum_size = Vector2(200, 30)
	server_url_input.placeholder_text = "ws://localhost:3000/ws"
	server_url_input.text = "ws://localhost:3000/ws"
	server_url_input.add_theme_font_size_override("font_size", 12)
	add_child(server_url_input)

	# Player ID input
	player_id_input = LineEdit.new()
	player_id_input.position = Vector2(BOARD_SIZE + 40, 390)
	player_id_input.custom_minimum_size = Vector2(200, 30)
	player_id_input.placeholder_text = "Player name"
	player_id_input.text = "player_" + str(randi() % 10000)
	player_id_input.add_theme_font_size_override("font_size", 12)
	add_child(player_id_input)

	# Connect button
	connect_button = Button.new()
	connect_button.position = Vector2(BOARD_SIZE + 40, 430)
	connect_button.custom_minimum_size = Vector2(200, 40)
	connect_button.text = "Connect & Join Queue"
	connect_button.add_theme_font_size_override("font_size", 14)
	connect_button.pressed.connect(_on_connect_button_pressed)
	add_child(connect_button)

	# Network status label
	network_status_label = Label.new()
	network_status_label.position = Vector2(BOARD_SIZE + 40, 480)
	network_status_label.custom_minimum_size = Vector2(200, 30)
	network_status_label.text = "Disconnected"
	network_status_label.add_theme_font_size_override("font_size", 14)
	network_status_label.add_theme_color_override("font_color", Color(0.7, 0.7, 0.7))
	add_child(network_status_label)

	# Game info label (opponent, color, game ID)
	game_info_label = Label.new()
	game_info_label.position = Vector2(BOARD_SIZE + 40, 510)
	game_info_label.custom_minimum_size = Vector2(200, 80)
	game_info_label.text = ""
	game_info_label.add_theme_font_size_override("font_size", 12)
	game_info_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	add_child(game_info_label)

func _connect_network_signals():
	network_manager.connection_established.connect(_on_connection_established)
	network_manager.connection_failed.connect(_on_connection_failed)
	network_manager.matchmaking_joined.connect(_on_matchmaking_joined)
	network_manager.match_found.connect(_on_match_found)
	network_manager.game_state_updated.connect(_on_game_state_updated)
	network_manager.opponent_action_received.connect(_on_opponent_action_received)
	network_manager.game_over.connect(_on_game_over)
	network_manager.invalid_action.connect(_on_invalid_action)
	network_manager.error_received.connect(_on_error_received)

func update_clock_display():
	if chess_game.has_clock():
		var white_time = chess_game.get_white_time()
		var black_time = chess_game.get_black_time()

		white_clock_label.text = format_time(white_time)
		black_clock_label.text = format_time(black_time)

		# Highlight active player's clock
		var current_turn = chess_game.get_current_turn()
		if current_turn == "white":
			white_clock_label.add_theme_color_override("font_color", Color(1.0, 1.0, 0.0))  # Yellow
			black_clock_label.add_theme_color_override("font_color", Color(1.0, 1.0, 1.0))  # White
		else:
			white_clock_label.add_theme_color_override("font_color", Color(1.0, 1.0, 1.0))  # White
			black_clock_label.add_theme_color_override("font_color", Color(1.0, 1.0, 0.0))  # Yellow
	else:
		white_clock_label.text = ""
		black_clock_label.text = ""

func format_time(seconds: int) -> String:
	if seconds < 0:
		return ""

	var mins = seconds / 60
	var secs = seconds % 60
	return "%d:%02d" % [mins, secs]

func _on_clock_tick():
	if not chess_game.is_game_over():
		var still_has_time = chess_game.tick_clock()
		update_clock_display()

		if not still_has_time:
			# Player ran out of time
			update_status()
			clock_timer.stop()
	else:
		clock_timer.stop()

func _draw():
	# Draw the chess board squares (flipped so white is at bottom)
	for row in range(8):
		for col in range(8):
			var is_light = (row + col) % 2 == 0
			var color = LIGHT_SQUARE_COLOR if is_light else DARK_SQUARE_COLOR
			var display_row = 7 - row  # Flip vertically
			var rect = Rect2(
				Vector2(20 + col * SQUARE_SIZE, 60 + display_row * SQUARE_SIZE),
				Vector2(SQUARE_SIZE, SQUARE_SIZE)
			)
			draw_rect(rect, color)

	# Highlight selected square
	if selected_square.x >= 0 and selected_square.y >= 0:
		var display_row = 7 - selected_square.x  # Flip vertically
		var rect = Rect2(
			Vector2(20 + selected_square.y * SQUARE_SIZE, 60 + display_row * SQUARE_SIZE),
			Vector2(SQUARE_SIZE, SQUARE_SIZE)
		)
		draw_rect(rect, SELECTED_COLOR)

	# Highlight legal move squares
	for i in range(0, legal_moves.size(), 2):
		var row = legal_moves[i]
		var col = legal_moves[i + 1]
		var display_row = 7 - row  # Flip vertically
		var rect = Rect2(
			Vector2(20 + col * SQUARE_SIZE, 60 + display_row * SQUARE_SIZE),
			Vector2(SQUARE_SIZE, SQUARE_SIZE)
		)
		draw_rect(rect, LEGAL_MOVE_COLOR)

	# Draw dragged piece following the cursor
	if is_dragging and drag_start_square.x >= 0:
		var piece = chess_game.get_piece_at(drag_start_square.x, drag_start_square.y)
		var piece_color = chess_game.get_piece_color_at(drag_start_square.x, drag_start_square.y)

		if piece != "":
			# Create a font for rendering the dragged piece
			var font = get_theme_default_font()
			var font_size = 48

			# Calculate position centered on cursor
			var text_size = font.get_string_size(piece, HORIZONTAL_ALIGNMENT_CENTER, -1, font_size)
			var draw_pos = drag_mouse_pos - text_size / 2

			# Draw piece with slight transparency to show it's being dragged
			var color = PEARL_WHITE_COLOR if piece_color == "white" else BLACK_COLOR
			color.a = 0.8  # Slight transparency
			draw_string(font, draw_pos, piece, HORIZONTAL_ALIGNMENT_CENTER, -1, font_size, color)

func update_board():
	if is_online_mode:
		# In online mode, use piece tracking by ID
		update_board_from_tracked_pieces()
	else:
		# In local mode, use traditional grid-based rendering
		update_board_local()

func update_board_local():
	"""Update board display in local mode (no piece tracking needed)"""
	for row in range(8):
		for col in range(8):
			var piece = chess_game.get_piece_at(row, col)
			var piece_color = chess_game.get_piece_color_at(row, col)

			# Hide piece if it's being dragged
			if is_dragging and drag_start_square.x == row and drag_start_square.y == col:
				piece_labels[row][col].text = ""
			else:
				piece_labels[row][col].text = piece

			# Apply color based on piece color
			if piece_color == "white":
				piece_labels[row][col].add_theme_color_override("font_color", PEARL_WHITE_COLOR)
			elif piece_color == "black":
				piece_labels[row][col].add_theme_color_override("font_color", BLACK_COLOR)

	queue_redraw()

func update_board_from_tracked_pieces():
	"""Update board display from tracked pieces (online mode)"""
	# Clear all piece labels first
	for row in range(8):
		for col in range(8):
			piece_labels[row][col].text = ""

	# Place pieces based on pieces_by_id
	for piece_id in pieces_by_id.keys():
		var piece_info = pieces_by_id[piece_id]
		var pos = piece_info["position"]
		var piece_type = piece_info["piece_type"]
		var color = piece_info["color"]

		if pos.x >= 0 and pos.x < 8 and pos.y >= 0 and pos.y < 8:
			# Get the symbol for this piece type
			var symbol = _get_piece_symbol(piece_type)

			# Hide piece if it's being dragged
			if is_dragging and drag_start_square.x == pos.x and drag_start_square.y == pos.y:
				piece_labels[pos.x][pos.y].text = ""
			else:
				piece_labels[pos.x][pos.y].text = symbol

			# Apply color
			if color == "white":
				piece_labels[pos.x][pos.y].add_theme_color_override("font_color", PEARL_WHITE_COLOR)
			else:
				piece_labels[pos.x][pos.y].add_theme_color_override("font_color", BLACK_COLOR)

	queue_redraw()

func _get_piece_symbol(piece_type: String) -> String:
	"""Get the unicode symbol for a piece type"""
	match piece_type.to_lower():
		"king":
			return "♚"
		"queen":
			return "♛"
		"rook":
			return "♜"
		"bishop":
			return "♝"
		"knight":
			return "♞"
		"pawn":
			return "♟"
		_:
			return "?"

func update_status():
	var status = chess_game.get_game_status()
	var turn = chess_game.get_current_turn()

	match status:
		"checkmate_white":
			status_label.text = "Checkmate! White wins!"
		"checkmate_black":
			status_label.text = "Checkmate! Black wins!"
		"timeloss_white":
			status_label.text = "Time out! Black wins!"
		"timeloss_black":
			status_label.text = "Time out! White wins!"
		"stalemate":
			status_label.text = "Stalemate! Draw."
		"draw":
			status_label.text = "Draw by insufficient material."
		"check":
			status_label.text = "Check! %s to move" % turn.capitalize()
		_:
			status_label.text = "%s to move" % turn.capitalize()

func _gui_input(event):
	# Check if game is over
	if chess_game.is_game_over():
		return
    
  # In online mode, check if it's our turn
	if is_online_mode:
		var current_turn = chess_game.get_current_turn()
		if current_turn != my_color:
			return  # Not our turn

	# Handle right-click to cancel selection/dragging
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_RIGHT and event.pressed:
		cancel_selection()
		return

	# Handle mouse button press (start drag or click)
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		var local_pos = event.position - Vector2(20, 60)

		# Check if within board bounds
		if local_pos.x < 0 or local_pos.y < 0 or local_pos.x >= BOARD_SIZE or local_pos.y >= BOARD_SIZE:
			return

		var col = int(local_pos.x / SQUARE_SIZE)
		var display_row = int(local_pos.y / SQUARE_SIZE)
		var row = 7 - display_row  # Convert display row to actual board row

		if event.pressed:
			# Mouse button pressed - start potential drag
			DebugUtils.debug_var("Mouse button pressed at", Vector2i(row, col))
			var piece = chess_game.get_piece_at(row, col)
			var piece_color = chess_game.get_piece_color_at(row, col)
			var current_turn = chess_game.get_current_turn()

			# Only start drag if there's a piece of the current player's color
			if piece != "" and piece_color == current_turn:
				is_dragging = true
				drag_start_square = Vector2i(row, col)
				drag_mouse_pos = event.position

				# Select the piece
				chess_game.select_piece(row, col)
				selected_square = Vector2i(row, col)
				legal_moves = chess_game.get_legal_moves_for_selected()
				queue_redraw()
		else:
			# Mouse button released - complete drag or click
			if is_dragging:
				DebugUtils.debug("Mouse button released, completing drag")
				# Try to move the piece to the drop location
				handle_piece_drop(row, col)
				is_dragging = false
				drag_start_square = Vector2i(-1, -1)
				queue_redraw()
			else:
				# Regular click (not a drag)
				handle_square_click(row, col)

	# Handle mouse motion (for dragging)
	elif event is InputEventMouseMotion and is_dragging:
		drag_mouse_pos = event.position
		queue_redraw()

func handle_piece_drop(row: int, col: int):
	DebugUtils.debug_vars({"handle_piece_drop row": row, "col": col})

	# In online mode, send the move to the server
	if is_online_mode:
		# Validate the move locally first
		if chess_game.try_move_selected(row, col):
			DebugUtils.debug("Drop successful - sending move to server")
			# Send the move to the server
			network_manager.submit_move(drag_start_square.x, drag_start_square.y, row, col, "")
			selected_square = Vector2i(-1, -1)
			legal_moves.clear()
			# Don't update board yet - wait for server confirmation
		else:
			DebugUtils.debug("Drop failed - invalid move")
			chess_game.deselect_piece()
			selected_square = Vector2i(-1, -1)
			legal_moves.clear()
			update_board()  # Restore visual state after invalid move
	else:
		# Local mode - update immediately
		if chess_game.try_move_selected(row, col):
			DebugUtils.debug("Drop successful - piece moved!")
			selected_square = Vector2i(-1, -1)
			legal_moves.clear()
			update_board()
			update_status()
			update_clock_display()
			check_ai_turn()
		else:
			DebugUtils.debug("Drop failed - invalid move")
			chess_game.deselect_piece()
			selected_square = Vector2i(-1, -1)
			legal_moves.clear()
			update_board()  # Restore visual state after invalid move

func cancel_selection():
	DebugUtils.debug("Cancelling selection/drag")

	if is_dragging:
		is_dragging = false
		drag_start_square = Vector2i(-1, -1)

	chess_game.deselect_piece()
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()

	update_board()

func handle_square_click(row: int, col: int):
	DebugUtils.debug_vars({"handle_square_click row": row, "col": col})
	DebugUtils.debug_var("Current selected_square", selected_square)

	# If we have a piece selected, try to move it
	if selected_square.x >= 0:
		DebugUtils.debug("Piece already selected, trying to move to (%d, %d)" % [row, col])

		# Check if this is a promotion move
		if chess_game.is_promotion_move(row, col):
			# Show promotion dialog
			DebugUtils.debug("This is a promotion move, showing dialog")
			pending_promotion_move = Vector2i(row, col)
			promotion_panel.visible = true
			return

		# In online mode, send the move to the server
		if is_online_mode:
			if chess_game.try_move_selected(row, col):
				DebugUtils.debug("Move successful - sending to server!")
				network_manager.submit_move(selected_square.x, selected_square.y, row, col, "")
				selected_square = Vector2i(-1, -1)
				legal_moves.clear()
				# Wait for server confirmation before updating
			else:
				DebugUtils.debug("Move failed, trying to select new piece")
				chess_game.deselect_piece()
				if chess_game.select_piece(row, col):
					DebugUtils.debug("New piece selected at (%d, %d)" % [row, col])
					selected_square = Vector2i(row, col)
					legal_moves = chess_game.get_legal_moves_for_selected()
					DebugUtils.debug_var("Legal moves", legal_moves)
					update_board()
					queue_redraw()
				else:
					DebugUtils.debug("No piece to select at (%d, %d)" % [row, col])
					selected_square = Vector2i(-1, -1)
					legal_moves.clear()
					update_board()
					queue_redraw()
		else:
			# Local mode
			if chess_game.try_move_selected(row, col):
				# Move was successful
				DebugUtils.debug("Move successful!")
				selected_square = Vector2i(-1, -1)
				legal_moves.clear()
				update_board()
				update_status()
				update_clock_display()
				check_ai_turn()
			else:
				DebugUtils.debug("Move failed, trying to select new piece")
				# Try to select the clicked square instead
				chess_game.deselect_piece()
				if chess_game.select_piece(row, col):
					DebugUtils.debug("New piece selected at (%d, %d)" % [row, col])
					selected_square = Vector2i(row, col)
					legal_moves = chess_game.get_legal_moves_for_selected()
					DebugUtils.debug_var("Legal moves", legal_moves)
					update_board()
					queue_redraw()
				else:
					DebugUtils.debug("No piece to select at (%d, %d)" % [row, col])
					selected_square = Vector2i(-1, -1)
					legal_moves.clear()
					update_board()
					queue_redraw()
	else:
		DebugUtils.debug("No piece selected, trying to select piece at (%d, %d)" % [row, col])
		# Try to select a piece
		if chess_game.select_piece(row, col):
			DebugUtils.debug("Piece selected successfully!")
			selected_square = Vector2i(row, col)
			legal_moves = chess_game.get_legal_moves_for_selected()
			DebugUtils.debug_var("Legal moves for selected piece", legal_moves)
			queue_redraw()
		else:
			DebugUtils.debug("Failed to select piece (empty square or wrong color)")

func _on_promotion_selected(piece_type: String):
	DebugUtils.debug_var("Promotion piece selected", piece_type)

	# Hide the promotion panel
	promotion_panel.visible = false

	# Execute the promotion move
	if pending_promotion_move.x >= 0:
		if is_online_mode:
			# Online mode - send promotion move to server
			if chess_game.try_move_selected_with_promotion(pending_promotion_move.x, pending_promotion_move.y, piece_type):
				DebugUtils.debug("Promotion move successful - sending to server!")
				network_manager.submit_move(selected_square.x, selected_square.y, pending_promotion_move.x, pending_promotion_move.y, piece_type)
				selected_square = Vector2i(-1, -1)
				legal_moves.clear()
				pending_promotion_move = Vector2i(-1, -1)
			else:
				DebugUtils.debug("Promotion move failed!")
				pending_promotion_move = Vector2i(-1, -1)
		else:
			# Local mode
			if chess_game.try_move_selected_with_promotion(pending_promotion_move.x, pending_promotion_move.y, piece_type):
				DebugUtils.debug("Promotion move successful!")
				selected_square = Vector2i(-1, -1)
				legal_moves.clear()
				pending_promotion_move = Vector2i(-1, -1)
				update_board()
				update_status()
			else:
				DebugUtils.debug("Promotion move failed!")
				pending_promotion_move = Vector2i(-1, -1)

func _on_reset_button_pressed():
	chess_game.reset_game()
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()
	update_board()
	update_status()
	check_ai_turn()

func _on_ai_toggle_changed(is_checked: bool):
	ai_plays_black = is_checked
	check_ai_turn()

func _on_clock_preset_changed(index: int):
	apply_clock_preset(index)

func apply_clock_preset(preset_index: int):
	var initial_time: int
	var increment: int

	match preset_index:
		0:  # "1 min"
			initial_time = 60
			increment = 0
		1:  # "3 | 2"
			initial_time = 180
			increment = 2
		2:  # "5 min"
			initial_time = 300
			increment = 0
		3:  # "10 min"
			initial_time = 600
			increment = 0
		4:  # "15 | 10"
			initial_time = 900
			increment = 10
		5:  # "30 min"
			initial_time = 1800
			increment = 0
		_:  # Default to "3 | 2"
			initial_time = 180
			increment = 2

	chess_game.reset_game_with_clock(initial_time, increment)
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()
	update_board()
	update_status()
	update_clock_display()

	# Restart clock timer
	if clock_timer:
		clock_timer.start()

	check_ai_turn()

func check_ai_turn():
	# If it's the AI's turn and game is not over, make an AI move
	if ai_plays_black and chess_game.get_current_turn() == "black" and not chess_game.is_game_over():
		make_ai_move()

func make_ai_move():
	# Small delay to make the AI move visible
	await get_tree().create_timer(0.3).timeout

	if chess_game.make_ai_move():
		update_board()
		update_status()
		update_clock_display()

# Network signal handlers
func _on_connect_button_pressed():
	if network_manager.is_server_connected():
		# Disconnect
		network_manager.disconnect_from_server()
		connect_button.text = "Connect & Join Queue"
		network_status_label.text = "Disconnected"
		network_status_label.add_theme_color_override("font_color", Color(0.7, 0.7, 0.7))
		is_online_mode = false
		game_info_label.text = ""
	else:
		# Connect
		var server_url = server_url_input.text
		var player_name = player_id_input.text
		if server_url == "" or player_name == "":
			network_status_label.text = "Enter server URL and player name"
			network_status_label.add_theme_color_override("font_color", Color(1.0, 0.3, 0.3))
			return

		network_manager.connect_to_server(server_url, player_name)
		network_status_label.text = "Connecting..."
		network_status_label.add_theme_color_override("font_color", Color(1.0, 1.0, 0.0))

func _on_connection_established():
	DebugUtils.debug("Connection established!")
	network_status_label.text = "Connected"
	network_status_label.add_theme_color_override("font_color", Color(0.3, 1.0, 0.3))
	connect_button.text = "Disconnect"

	# Join matchmaking automatically
	network_manager.join_matchmaking()

func _on_connection_failed(reason: String):
	DebugUtils.debug_var("Connection failed", reason)
	network_status_label.text = "Connection failed: " + reason
	network_status_label.add_theme_color_override("font_color", Color(1.0, 0.3, 0.3))
	is_online_mode = false

func _on_matchmaking_joined():
	DebugUtils.debug("Matchmaking joined")
	network_status_label.text = "In matchmaking queue..."
	network_status_label.add_theme_color_override("font_color", Color(0.3, 0.8, 1.0))

func _on_match_found(game_id: String, opponent_id: String, your_color: String):
	DebugUtils.debug_vars({"Match found - Game ID": game_id, "Opponent": opponent_id, "Your color": your_color})
	is_online_mode = true
	my_color = your_color

	network_status_label.text = "Match found!"
	network_status_label.add_theme_color_override("font_color", Color(0.3, 1.0, 0.3))

	game_info_label.text = "Game: " + game_id.substr(0, 8) + "...\nOpponent: " + opponent_id + "\nYou are: " + your_color.capitalize()

	# Reset the game for online play
	chess_game.reset_game()
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()
	update_board()
	update_status()

func _on_game_state_updated(state: Dictionary):
	DebugUtils.debug("Game state updated from server")

	# Check if there's an action to animate
	var last_action = state.get("last_action", null)

	if last_action != null:
		# Animate the action first, then reconcile
		_animate_action_and_reconcile(state, last_action)
	else:
		# No action to animate, just apply the state directly
		_apply_game_state_immediately(state)

func _apply_game_state_immediately(state: Dictionary):
	"""Apply game state without animation"""
	# Update the board with the server's state
	var board_state = state.get("board_state", [])
	if board_state.size() > 0:
		_apply_server_board_state(board_state)

	# Update turn state from server
	var next_player_id = state.get("next_player_id", "")
	if next_player_id != "":
		_apply_turn_state(next_player_id)

	# Update timers
	var time_state = state.get("time", {})
	_apply_server_time_state(time_state)

	# Clear any selected pieces since the board has been updated
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()
	chess_game.deselect_piece()

	update_board()
	update_status()
	update_clock_display()

func _animate_action_and_reconcile(state: Dictionary, action: Dictionary):
	"""Animate an action and then reconcile with server state"""
	DebugUtils.debug_var("Animating action", action)

	# For now, implement a simple version that just applies the state
	# TODO: Implement actual animations based on action type
	_apply_game_state_immediately(state)

	# Future: Add animation logic here based on action type:
	# - Move: Tween piece from 'from' to 'to'
	# - Capture: Fade out victim, then tween attacker
	# - Castle: Tween both king and rook simultaneously
	# - EnPassant: Tween pawn to capture square, fade out captured pawn
	# - Promotion: Fade out old pawn (old_pawn_id), fade in new piece (new_piece_id)
	#   Note: Promotion creates a NEW piece with a NEW ID - the pawn is destroyed
	# After animations complete, reconciliation ensures visual matches server state

func _on_opponent_action_received(action: Dictionary):
	DebugUtils.debug_var("Opponent action received", action)
	# The game state update will follow, so we just need to wait for that

func _on_game_over(winner: String, reason: String):
	DebugUtils.debug_vars({"Game over - Winner": winner, "Reason": reason})
	var winner_text = winner if winner != "" else "Draw"
	network_status_label.text = "Game over: " + winner_text
	network_status_label.add_theme_color_override("font_color", Color(1.0, 0.8, 0.3))

	game_info_label.text = "Game over!\nWinner: " + winner_text + "\nReason: " + reason

	is_online_mode = false
	my_color = ""

func _on_invalid_action(reason: String):
	DebugUtils.debug_var("Invalid action", reason)
	status_label.text = "Invalid move: " + reason
	status_label.add_theme_color_override("font_color", Color(1.0, 0.3, 0.3))

	# Restore game state after server rejection
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()
	chess_game.deselect_piece()
	update_board()

	# Clear error message after a short delay
	await get_tree().create_timer(2.0).timeout
	update_status()

func _on_error_received(message: String):
	DebugUtils.debug_var("Error from server", message)
	network_status_label.text = "Error: " + message
	network_status_label.add_theme_color_override("font_color", Color(1.0, 0.3, 0.3))

func _apply_server_board_state(board_state: Array):
	"""
	Apply server's ID-based board state to local game with reconciliation.
	Format: [{id: u8, position: "e4", piece_type: "pawn"}, ...]
	IDs 0-15 are White, 16-31 are Black

	Reconciliation automatically handles:
	- Pieces that moved (update position)
	- Pieces that were captured (removed from board_state -> removed locally)
	- Pieces that were created (promotions create new IDs -> added locally)
	- Pieces that changed type (promotions destroy old pawn, create new piece with new ID)
	"""
	DebugUtils.debug_var("Applying server board state", board_state)

	# Clear the local chess_game board representation
	for row in range(8):
		for col in range(8):
			chess_game.clear_square(row, col)

	# Clear en passant target when reconstructing board from server state
	chess_game.clear_en_passant_target()

	# Track which pieces we've seen in this update
	var seen_piece_ids = {}

	# Apply each piece from the server state
	for piece_state in board_state:
		var piece_id = piece_state.get("id", -1)
		var position = piece_state.get("position", "")
		var piece_type = piece_state.get("piece_type", "")

		if piece_id < 0 or position == "" or piece_type == "":
			continue

		seen_piece_ids[piece_id] = true

		# Determine color from ID
		var is_white = piece_id < 16
		var color = "white" if is_white else "black"

		# Convert algebraic position to board coordinates
		var pos_vec = _algebraic_to_position(position)
		if pos_vec != null:
			# Update chess_game board state
			chess_game.place_piece(pos_vec.x, pos_vec.y, piece_type, color, piece_id)

			# Update or create piece tracking
			if not pieces_by_id.has(piece_id):
				# New piece - will be created during update_board
				pieces_by_id[piece_id] = {
					"position": pos_vec,
					"piece_type": piece_type,
					"color": color,
					"label": null
				}
			else:
				# Existing piece - update position
				pieces_by_id[piece_id]["position"] = pos_vec
				pieces_by_id[piece_id]["piece_type"] = piece_type

	# Remove pieces that no longer exist on the server
	var pieces_to_remove = []
	for piece_id in pieces_by_id.keys():
		if not seen_piece_ids.has(piece_id):
			pieces_to_remove.append(piece_id)

	for piece_id in pieces_to_remove:
		if pieces_by_id[piece_id]["label"] != null:
			pieces_by_id[piece_id]["label"].queue_free()
		pieces_by_id.erase(piece_id)

	DebugUtils.debug("Board state applied successfully")

func _apply_turn_state(next_player_id: String):
	"""
	Apply server's turn state to local game.
	Converts next_player_id to a color and sets it.
	"""
	DebugUtils.debug_var("Applying turn state - next player", next_player_id)

	# Determine which color should move next
	var next_color: String
	if next_player_id == network_manager.player_id:
		# It's this player's turn
		next_color = my_color
	else:
		# It's the opponent's turn
		next_color = "white" if my_color == "black" else "black"

	DebugUtils.debug_var("Setting current turn to", next_color)
	chess_game.set_current_turn(next_color)

func _apply_server_time_state(time_state: Dictionary):
	"""
	Apply server's time state to local game.
	Format: {player_id: seconds_remaining, ...}
	"""
	if time_state.size() == 0:
		return

	DebugUtils.debug_var("Applying time state", time_state)

	for player_id in time_state.keys():
		var time_remaining = time_state[player_id]
		var is_white = (player_id == network_manager.player_id and my_color == "white") or \
		               (player_id != network_manager.player_id and my_color == "black")

		if is_white:
			chess_game.set_white_time(time_remaining)
		else:
			chess_game.set_black_time(time_remaining)

func _algebraic_to_position(algebraic: String):
	"""
	Convert algebraic notation (e.g., "a1") to board position (row, col).
	Returns Vector2i or null if invalid.
	"""
	if algebraic.length() != 2:
		return null

	var col = algebraic.unicode_at(0) - "a".unicode_at(0)
	var row = algebraic.unicode_at(1) - "1".unicode_at(0)

	if row < 0 or row > 7 or col < 0 or col > 7:
		return null

	return Vector2i(row, col)
