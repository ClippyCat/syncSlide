# Adapted from simple_websocket demo: https://simple-websocket.readthedocs.io/en/latest/intro.html#server-example-2-aiohttp

from aiohttp import web
from simple_websocket import AioServer, ConnectionClosed

app = web.Application()

clients = []

def add_client(ws):
	uid = 0
	if len(clients) != 0:
		uid = max([x for (x,y) in clients])
	uid += 1
	clients.append((uid, ws))
	return uid

def remove_client(uid):
	idx = [i for (i,(x,y)) in enumerate(clients) if x == uid]
	if len(idx) == 1:
		del clients[idx[0]]

async def broadcast_to_all(request):
	ws = await AioServer.accept(aiohttp=request)
	client_idx = add_client(ws)
	try:
		while True:
# wait for this specific client to send a message to the server
			data = await ws.receive()
# send each and every client the same message
			for (_,client) in clients:
				await client.send(data)
# if client disconnected, do nothing
	except ConnectionClosed:
		remove_client(client_idx)
# must return a valid HTTP response, even if it is a blank string
	return web.Response(text='')

# route broadcast_to_all to root (/) URL.
app.add_routes([web.get('/', broadcast_to_all)])

# if the file is being run from the command line
if __name__ == '__main__':
	web.run_app(app, port=5000)
