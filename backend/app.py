from flask import Flask, render_template
from flask_socketio import SocketIO
from flask_socketio import send, emit
app = Flask(__name__)
app.config['SECRET_KEY'] = 'secret!'
socketio = SocketIO(app)

@socketio.on('message')
def handle_message(message):
    send("message", broadcast=True)

if __name__ == '__main__':
socketio.run(app)