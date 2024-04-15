# Adapted from simple_websocket demo: https://simple-websocket.readthedocs.io/en/latest/intro.html#server-example-2-aiohttp

from aiohttp import web
from simple_websocket import AioServer, ConnectionClosed

app = web.Application()

clients = []

async def broadcast_to_all(request):
	ws = await AioServer.accept(aiohttp=request)
	clients.append(ws)
	try:
		while True:
# wait for this specific client to send a message to the server
			data = await ws.receive()
# send each and every client the same message
			for client in clients:
				await client.send(data)
# if client disconnected, do nothing
	except ConnectionClosed:
		clients = [x for x in clients if x != ws]
# must return a valid HTTP response, even if it is a blank string
	return web.Response(text='')

# route broadcast_to_all to root (/) URL.
app.add_routes([web.get('/', broadcast_to_all)])

# if the file is being run from the command line
if __name__ == '__main__':
	web.run_app(app, port=5000)
