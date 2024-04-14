# `syncSlide` server

## Installation

To get started, open a virtual environment, then run using `python`

### Linux Steps

- `python -mvenv env`
- `source env/bin/activate`
- `pip install -r requirements.txt`
- `python main.py`

### Windows Steps

- `python -mvenv env`
- `venv\Scripts\Activate.ps1`
- `pip install -r requirements.txt`
- `python main.py`

## How To Test

Using a websocket CLI client, open two teminal windows, and run them using these URLs:

- `websocat ws://127.0.0.1:5000/`
