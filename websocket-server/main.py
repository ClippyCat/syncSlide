# Adapted from simple_websocket demo: https://simple-websocket.readthedocs.io/en/latest/intro.html#server-example-2-aiohttp

import json, signal
from aiohttp import web
from simple_websocket import AioServer, ConnectionClosed
import jinja2
import aiohttp_jinja2 as aj


app = web.Application()

SLIDE_CONTENTS = dict()
SLIDE_IDX = dict()

clients = dict()

def cleanup(_signame, _stackframe):
	print("Cleaning up abandoned presentations...", end="")
	for pid in SLIDE_CONTENTS.keys():
		if pid not in clients:
			del SLIDE_CONTENTS[pid]
		if pid not in SLIDE_IDX:
			del SLIDE_IDX[pid]
	print("done")

def add_client(ws, pid):
	uid = 0
	if pid not in clients:
		clients[pid] = []
	if len(clients[pid]) != 0:
		uid = max([x for (x,y) in clients[pid]])
	uid += 1
	clients[pid].append((uid, ws))
	return uid

def remove_client(uid, pid):
	idx = [i for (i,(x,y)) in enumerate(clients[pid]) if x == uid]
	if len(idx) == 1:
		del clients[pid][idx[0]]

# no need for async since we are not waiting on sending/receiving data
# pid = presentation id
def update_active_slide_data(raw_msg, pid):
	# use global vars instead of creating new, local vars
	global SLIDE_CONTENTS
	global SLIDE_IDX
	# BTW json is a module, this vairbale can not also be called json
	json_msg = json.loads(raw_msg)
	if json_msg["type"] == "text":
		SLIDE_CONTENTS[pid] = json_msg["text"]
	elif json_msg["type"] == "slide":
		SLIDE_IDX[pid] = json_msg["slide"]

async def send_active_slide_data(ws, pid):
	global SLIDE_CONTENTS
	global SLIDE_IDX
	if pid not in SLIDE_CONTENTS or not SLIDE_CONTENTS[pid] or not SLIDE_IDX[pid]:
		return
	await ws.send(json.dumps({"type": "text", "text": SLIDE_CONTENTS[pid]}))
	await ws.send(json.dumps({"type": "slide", "slide": SLIDE_IDX[pid]}))

async def broadcast_to_all(request):
	pid = request.match_info["pid"]
	ws = await AioServer.accept(aiohttp=request)
	client_idx = add_client(ws, pid)
	await send_active_slide_data(ws, pid)
	try:
		while True:
# wait for this specific client to send a message to the server
			data = await ws.receive()
			update_active_slide_data(data, pid)
# send each and every client the same message
			for (_,client) in clients[pid]:
				await client.send(data)
# if client disconnected, do nothing
	except ConnectionClosed:
		remove_client(client_idx, pid)
# must return a valid HTTP response, even if it is a blank string
	return web.Response(text='')

@aj.template("join.html")
async def join(request):
	return {}

@aj.template("start.html")
async def start(request):
	return {}

@aj.template("audience.html")
async def audience(request):
	return {}

@aj.template("stage.html")
async def stage(request):
	return {}

aj.setup(app, loader=jinja2.FileSystemLoader("templates"))

# route broadcast_to_all to root (/) URL.
app.add_routes([
	web.static('/js', '../src/js/'),
	web.static('/css', '../src/css/'),
	web.get('/audience', join),
	web.get('/audience/{pid}', audience),
	web.get('/stage', start),
	web.get('/stage/{pid}', stage),
	web.get('/ws/{pid}', broadcast_to_all),
])

# if the file is being run from the command line
if __name__ == '__main__':
	signal.signal(signal.SIGUSR1, cleanup)
	web.run_app(app, port=5002)
