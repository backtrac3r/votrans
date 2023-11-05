import argparse
from telethon import TelegramClient

parser = argparse.ArgumentParser(description='Optional app description')

parser.add_argument('--file_id', type=str, help='file_id')
parser.add_argument('--api_id', type=int, help='api_id')
parser.add_argument('--api_hash', type=str, help='api_hash')

args = parser.parse_args()

client = TelegramClient('anon', args.api_id, args.api_hash)

async def main():
    file_bytes = await client.download_media(args.file_id, file=bytes)
    print(file_bytes)

with client:
    client.loop.run_until_complete(main())
