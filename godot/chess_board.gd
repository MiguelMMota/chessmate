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

# Game state
var chess_game: ChessGame
var selected_square: Vector2i = Vector2i(-1, -1)
var legal_moves: Array = []

# UI nodes
var board_container: Control
var piece_labels: Array = []
var status_label: Label

func _ready():
	DebugUtils.debug("ChessBoard _ready() called")

	# Enable mouse input for this Control node
	mouse_filter = Control.MOUSE_FILTER_STOP
	DebugUtils.debug("Mouse filter set to STOP")

	# Create the chess game instance
	chess_game = ChessGame.new()
	add_child(chess_game)
	DebugUtils.debug("ChessGame instance created and added as child")

	# Setup UI
	setup_board()
	setup_status_label()
	update_board()
	DebugUtils.debug("Board setup complete")

func setup_board():
	board_container = Control.new()
	board_container.position = Vector2(20, 60)
	board_container.custom_minimum_size = Vector2(BOARD_SIZE, BOARD_SIZE)
	# Allow mouse events to pass through to parent
	board_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(board_container)

	# Create labels for pieces
	for row in range(8):
		var row_array = []
		for col in range(8):
			var label = Label.new()
			label.position = Vector2(col * SQUARE_SIZE, row * SQUARE_SIZE)
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

func _draw():
	# Draw the chess board squares
	for row in range(8):
		for col in range(8):
			var is_light = (row + col) % 2 == 0
			var color = LIGHT_SQUARE_COLOR if is_light else DARK_SQUARE_COLOR
			var rect = Rect2(
				Vector2(20 + col * SQUARE_SIZE, 60 + row * SQUARE_SIZE),
				Vector2(SQUARE_SIZE, SQUARE_SIZE)
			)
			draw_rect(rect, color)

	# Highlight selected square
	if selected_square.x >= 0 and selected_square.y >= 0:
		var rect = Rect2(
			Vector2(20 + selected_square.y * SQUARE_SIZE, 60 + selected_square.x * SQUARE_SIZE),
			Vector2(SQUARE_SIZE, SQUARE_SIZE)
		)
		draw_rect(rect, SELECTED_COLOR)

	# Highlight legal move squares
	for i in range(0, legal_moves.size(), 2):
		var row = legal_moves[i]
		var col = legal_moves[i + 1]
		var rect = Rect2(
			Vector2(20 + col * SQUARE_SIZE, 60 + row * SQUARE_SIZE),
			Vector2(SQUARE_SIZE, SQUARE_SIZE)
		)
		draw_rect(rect, LEGAL_MOVE_COLOR)

func update_board():
	# Update all piece labels
	for row in range(8):
		for col in range(8):
			var piece = chess_game.get_piece_at(row, col)
			piece_labels[row][col].text = piece

	queue_redraw()

func update_status():
	var status = chess_game.get_game_status()
	var turn = chess_game.get_current_turn()

	match status:
		"checkmate_white":
			status_label.text = "Checkmate! White wins!"
		"checkmate_black":
			status_label.text = "Checkmate! Black wins!"
		"stalemate":
			status_label.text = "Stalemate! Draw."
		"draw":
			status_label.text = "Draw by insufficient material."
		"check":
			status_label.text = "Check! %s to move" % turn.capitalize()
		_:
			status_label.text = "%s to move" % turn.capitalize()

func _gui_input(event):
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		DebugUtils.debug_var("Mouse button left pressed at position", event.position)

		# Check if game is over
		if chess_game.is_game_over():
			DebugUtils.debug("Game is over, ignoring click")
			return

		# Convert mouse position to board coordinates
		var local_pos = event.position - Vector2(20, 60)
		DebugUtils.debug_var("Local position", local_pos)

		if local_pos.x < 0 or local_pos.y < 0 or local_pos.x >= BOARD_SIZE or local_pos.y >= BOARD_SIZE:
			DebugUtils.debug("Click outside board bounds")
			return

		var col = int(local_pos.x / SQUARE_SIZE)
		var row = int(local_pos.y / SQUARE_SIZE)
		DebugUtils.debug_vars({"Calculated row": row, "col": col})

		handle_square_click(row, col)

func handle_square_click(row: int, col: int):
	DebugUtils.debug_vars({"handle_square_click row": row, "col": col})
	DebugUtils.debug_var("Current selected_square", selected_square)

	# If we have a piece selected, try to move it
	if selected_square.x >= 0:
		DebugUtils.debug("Piece already selected, trying to move to (%d, %d)" % [row, col])
		if chess_game.try_move_selected(row, col):
			# Move was successful
			DebugUtils.debug("Move successful!")
			selected_square = Vector2i(-1, -1)
			legal_moves.clear()
			update_board()
			update_status()
		else:
			DebugUtils.debug("Move failed, trying to select new piece")
			# Try to select the clicked square instead
			chess_game.deselect_piece()
			if chess_game.select_piece(row, col):
				DebugUtils.debug("New piece selected at (%d, %d)" % [row, col])
				selected_square = Vector2i(row, col)
				legal_moves = chess_game.get_legal_moves_for_selected()
				DebugUtils.debug_var("Legal moves", legal_moves)
				queue_redraw()
			else:
				DebugUtils.debug("No piece to select at (%d, %d)" % [row, col])
				selected_square = Vector2i(-1, -1)
				legal_moves.clear()
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

func _on_reset_button_pressed():
	chess_game.reset_game()
	selected_square = Vector2i(-1, -1)
	legal_moves.clear()
	update_board()
	update_status()
