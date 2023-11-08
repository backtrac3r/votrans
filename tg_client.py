import asyncio
import argparse
from pyrogram import Client

parser = argparse.ArgumentParser(description="Optional app description")

parser.add_argument("--api_id", type=int, help="api_id")
parser.add_argument("--api_hash", type=str, help="api_hash")

args = parser.parse_args()

app = Client("anon", args.api_id, args.api_hash, max_concurrent_transmissions=3)

@app.on_message()
async def handler(client, message):
    await message.download(file_name="temp/")
    await message.reply("complete")

app.run()
