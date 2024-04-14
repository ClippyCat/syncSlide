from flask import Flask, request
from simple_websocket import Server, ConnectionClosed

app = Flask(__name__)
clients = []

# start using the '/echo' endpoint
@app.route('/echo', websocket=True)
def echo():
    ws = Server.accept(request.environ)
    # add to list of all clients
    clients.append(ws)
    # .receive() will raise an exception if nothing is there, that's why we have to try
    try:
        while True:
            data = ws.receive()
            # send each client the same message
            for client in clients:
                client.send(data)
    # if the connection is closed, just leave the handler (returning '' is valid)
    except ConnectionClosed:
        return ''
    # other exceptions also should return nothing
    return ''

