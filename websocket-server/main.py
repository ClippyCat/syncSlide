# Adapted from simple_websocket demo: https://simple-websocket.readthedocs.io/en/latest/intro.html#server-example-2-aiohttp

import json
from aiohttp import web
from simple_websocket import AioServer, ConnectionClosed
import jinja2
import aiohttp_jinja2 as aj


app = web.Application()

SLIDE_CONTENTS = None
SLIDE_IDX = None

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

# no need for async since we are not waiting on sending/receiving data
def update_active_slide_data(raw_msg):
	# use global vars instead of creating new, local vars
	global SLIDE_CONTENTS
	global SLIDE_IDX
	# BTW json is a module, this vairbale can not also be called json
	json_msg = json.loads(raw_msg)
	if json_msg["type"] == "text":
		SLIDE_CONTENTS = json_msg["text"]
	elif json_msg["type"] == "slide":
		SLIDE_IDX = json_msg["slide"]

async def send_active_slide_data(ws):
	global SLIDE_CONTENTS
	global SLIDE_IDX
	if not SLIDE_CONTENTS or not SLIDE_IDX:
		return
	await ws.send(json.dumps({"type": "text", "text": SLIDE_CONTENTS}))
	await ws.send(json.dumps({"type": "slide", "slide": SLIDE_IDX}))

async def broadcast_to_all(request):
	ws = await AioServer.accept(aiohttp=request)
	client_idx = add_client(ws)
	await send_active_slide_data(ws)
	try:
		while True:
# wait for this specific client to send a message to the server
			data = await ws.receive()
			update_active_slide_data(data)
# send each and every client the same message
			for (_,client) in clients:
				await client.send(data)
# if client disconnected, do nothing
	except ConnectionClosed:
		remove_client(client_idx)
# must return a valid HTTP response, even if it is a blank string
	return web.Response(text='')

@aj.template("audience.html")
async def audience(request):
	return {}

@aj.template("stage.html")
async def stage(request):
	return {}

aj.setup(app, loader=jinja2.FileSystemLoader("templates"))

# route broadcast_to_all to root (/) URL.
app.add_routes([
	web.get('/audience', audience),
	web.get('/stage', stage),
	web.get('/ws', broadcast_to_all),
])

# if the file is being run from the command line
if __name__ == '__main__':
	web.run_app(app, port=5002)
