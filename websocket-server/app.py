import aiohttp
from aiohttp import web
from authlib.integrations.starlette_client import OAuth
from aiohttp_session import setup, get_session, session_middleware
from aiohttp_session.cookie_storage import EncryptedCookieStorage
import base64
from cryptography.fernet import Fernet

# Generate a secret key
from cryptography.fernet import Fernet
secret_key = Fernet.generate_key()


# Initialize the app
app = web.Application()

# Setup session middleware
setup(app, EncryptedCookieStorage(base64.urlsafe_b64decode(secret_key)))

# Configure OAuth
oauth = OAuth()
oauth.register(
	name='google',
	client_id='YOUR_GOOGLE_CLIENT_ID',
	client_secret='YOUR_GOOGLE_CLIENT_SECRET',
	authorize_url='https://accounts.google.com/o/oauth2/auth',
	authorize_params=None,
	authorize_endpoint='https://accounts.google.com/o/oauth2/v2/auth',
	access_token_url='https://accounts.google.com/o/oauth2/token',
	access_token_params=None,
	userinfo_endpoint='https://openidconnect.googleapis.com/v1/userinfo',
	client_kwargs={'scope': 'openid profile email'}
)

# Middleware to add session to request
@web.middleware
async def session_middleware(request, handler):
	request.session = await get_session(request)
	response = await handler(request)
	
	if isinstance(response, web.HTTPFound):
		return response
	
	return response

app.middlewares.append(session_middleware)

async def index(request):
	session = await get_session(request)
	user = session.get('user')
	return web.Response(text=f"Hello, {user['name']}" if user else "Hello, please login: <a href='/login'>Login with Google</a>", content_type='text/html')

async def login(request):
	redirect_uri = str(request.url.with_path('/auth'))
	response = await oauth.google.authorize_redirect(request, redirect_uri)
	return response

async def auth(request):
	token = await oauth.google.authorize_access_token(request)
	user = await oauth.google.parse_id_token(request, token)
	session = await get_session(request)
	session['user'] = user
	return web.HTTPFound('/')

async def logout(request):
	session = await get_session(request)
	session.pop('user', None)
	return web.HTTPFound('/')

app.add_routes([web.get('/', index)])
app.add_routes([web.get('/login', login)])
app.add_routes([web.get('/auth', auth)])
app.add_routes([web.get('/logout', logout)])

if __name__ == '__main__':
	web.run_app(app, port=8080)
