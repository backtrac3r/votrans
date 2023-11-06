import asyncio
import argparse
from pyrogram import Client

parser = argparse.ArgumentParser(description="Optional app description")

parser.add_argument("--file_id", type=str, help="file_id")
parser.add_argument("--api_id", type=int, help="api_id")
parser.add_argument("--api_hash", type=str, help="api_hash")

args = parser.parse_args()

async def main():
    async with Client("anon", args.api_id, args.api_hash, max_concurrent_transmissions=2) as client:
        path = await client.download_media(args.file_id)
        print(path)


asyncio.run(main())
