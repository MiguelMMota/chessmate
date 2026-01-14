extends Node

# Debug utilities
const DebugUtils = preload("res://debug_utils.gd")

# Network state
enum NetworkState {
	DISCONNECTED,
	CONNECTING,
	CONNECTED,
	IN_MATCHMAKING,
	IN_GAME
}

# Signals
signal connection_established
signal connection_failed(reason: String)
signal matchmaking_joined
signal match_found(game_id: String, opponent_id: String, your_color: String)
signal game_state_updated(state: Dictionary)
signal opponent_action_received(action: Dictionary)
signal game_over(winner: String, reason: String)
signal invalid_action(reason: String)
signal error_received(message: String)

# WebSocket
var websocket: WebSocketPeer
var state: NetworkState = NetworkState.DISCONNECTED

# Connection info
var server_url: String = ""
var player_id: String = ""

# Game info
var current_game_id: String = ""
var your_color: String = ""
var opponent_id: String = ""

func _ready():
	websocket = WebSocketPeer.new()
	DebugUtils.debug("NetworkManager initialized")

func _process(_delta):
	if state == NetworkState.DISCONNECTED:
		return

	# Poll the WebSocket
	websocket.poll()

	var ws_state = websocket.get_ready_state()

	# Check connection state
	if ws_state == WebSocketPeer.STATE_OPEN:
		if state == NetworkState.CONNECTING:
			state = NetworkState.CONNECTED
			DebugUtils.debug("WebSocket connection established")
			connection_established.emit()

		# Process incoming messages
		while websocket.get_available_packet_count() > 0:
			var packet = websocket.get_packet()
			var json_string = packet.get_string_from_utf8()
			DebugUtils.debug_var("Received message", json_string)
			_handle_server_message(json_string)

	elif ws_state == WebSocketPeer.STATE_CLOSED:
		if state != NetworkState.DISCONNECTED:
			DebugUtils.debug("WebSocket connection closed")
			var close_code = websocket.get_close_code()
			var close_reason = websocket.get_close_reason()
			DebugUtils.debug_vars({"close_code": close_code, "close_reason": close_reason})
			disconnect_from_server()
			connection_failed.emit("Connection closed: " + close_reason)

func connect_to_server(url: String, player_name: String) -> void:
	if state != NetworkState.DISCONNECTED:
		DebugUtils.debug("Already connected or connecting")
		return

	server_url = url
	player_id = player_name

	DebugUtils.debug_vars({"Connecting to": server_url, "Player ID": player_id})

	state = NetworkState.CONNECTING
	var error = websocket.connect_to_url(server_url)

	if error != OK:
		DebugUtils.debug_var("Failed to connect", error)
		state = NetworkState.DISCONNECTED
		connection_failed.emit("Failed to initiate connection: " + str(error))

func disconnect_from_server() -> void:
	if state != NetworkState.DISCONNECTED:
		websocket.close()
		state = NetworkState.DISCONNECTED
		current_game_id = ""
		your_color = ""
		opponent_id = ""
		DebugUtils.debug("Disconnected from server")

func join_matchmaking() -> void:
	if state != NetworkState.CONNECTED:
		DebugUtils.debug("Not connected to server")
		return

	var message = {
		"type": "JoinMatchmaking",
		"player_id": player_id
	}
	_send_message(message)
	state = NetworkState.IN_MATCHMAKING
	DebugUtils.debug("Joining matchmaking queue")

func submit_move(from_row: int, from_col: int, to_row: int, to_col: int, promotion: String = "") -> void:
	if state != NetworkState.IN_GAME or current_game_id == "":
		DebugUtils.debug("Not in a game")
		return

	var action = {
		"action_type": "MovePiece",
		"from": {"row": from_row, "col": from_col},
		"to": {"row": to_row, "col": to_col}
	}

	if promotion != "":
		action["promotion"] = _piece_type_to_protocol(promotion)

	var message = {
		"type": "SubmitAction",
		"game_id": current_game_id,
		"action": action
	}

	DebugUtils.debug_var("Submitting move", message)
	_send_message(message)

func resign() -> void:
	if state != NetworkState.IN_GAME or current_game_id == "":
		DebugUtils.debug("Not in a game")
		return

	var action = {
		"action_type": "Resign"
	}

	var message = {
		"type": "SubmitAction",
		"game_id": current_game_id,
		"action": action
	}

	DebugUtils.debug("Resigning from game")
	_send_message(message)

func leave_game() -> void:
	if current_game_id == "":
		return

	var message = {
		"type": "LeaveGame",
		"game_id": current_game_id
	}

	_send_message(message)
	current_game_id = ""
	your_color = ""
	opponent_id = ""
	state = NetworkState.CONNECTED

func _send_message(message: Dictionary) -> void:
	var json_string = JSON.stringify(message)
	DebugUtils.debug_var("Sending message", json_string)
	websocket.send_text(json_string)

func _handle_server_message(json_string: String) -> void:
	var json = JSON.new()
	var parse_result = json.parse(json_string)

	if parse_result != OK:
		DebugUtils.debug_var("Failed to parse JSON", json_string)
		return

	var message = json.data
	var msg_type = message.get("type", "")

	DebugUtils.debug_var("Handling message type", msg_type)

	match msg_type:
		"MatchmakingJoined":
			DebugUtils.debug("Matchmaking joined")
			matchmaking_joined.emit()

		"MatchFound":
			current_game_id = message.get("game_id", "")
			opponent_id = message.get("opponent_id", "")
			your_color = message.get("your_color", "").to_lower()
			state = NetworkState.IN_GAME
			DebugUtils.debug_vars({
				"Match found - Game ID": current_game_id,
				"Opponent": opponent_id,
				"Your color": your_color
			})
			match_found.emit(current_game_id, opponent_id, your_color)

		"GameStateUpdate":
			var game_state = message.get("state", {})
			DebugUtils.debug_var("Game state update", game_state)
			game_state_updated.emit(game_state)

		"OpponentAction":
			var action = message.get("action", {})
			DebugUtils.debug_var("Opponent action", action)
			opponent_action_received.emit(action)

		"GameOver":
			var winner = message.get("winner", null)
			var reason = message.get("reason", "")
			DebugUtils.debug_vars({"Game over - Winner": winner, "Reason": reason})
			game_over.emit(str(winner) if winner != null else "", reason)
			current_game_id = ""
			your_color = ""
			opponent_id = ""
			state = NetworkState.CONNECTED

		"InvalidAction":
			var reason = message.get("reason", "")
			DebugUtils.debug_var("Invalid action", reason)
			invalid_action.emit(reason)

		"Error":
			var err_message = message.get("message", "")
			DebugUtils.debug_var("Error from server", err_message)
			error_received.emit(err_message)

func _piece_type_to_protocol(piece_type: String) -> String:
	match piece_type.to_lower():
		"queen":
			return "Queen"
		"rook":
			return "Rook"
		"bishop":
			return "Bishop"
		"knight":
			return "Knight"
		_:
			return "Queen"  # Default to queen

func is_server_connected() -> bool:
	return state != NetworkState.DISCONNECTED and state != NetworkState.CONNECTING

func is_in_game() -> bool:
	return state == NetworkState.IN_GAME

func get_state() -> NetworkState:
	return state
