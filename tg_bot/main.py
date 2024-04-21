import os
import subprocess
import json
import tempfile
import asyncio
import logging

from web3 import AsyncWeb3, AsyncHTTPProvider
import telebot
from telebot import asyncio_filters
from telebot.async_telebot import AsyncTeleBot
from telebot.asyncio_storage import StateMemoryStorage
from telebot.asyncio_handler_backends import State, StatesGroup
from telebot import apihelper
from telebot import formatting
import aioschedule

from config import (
    ODESEC_BIN, FULL_NODE_RPC_URL, BOT_TOKEN, ODESEC_CONTRACT_ADDRESS,
    DEFAULT_POC_ADDRESS, FAMOUS_BALANCE_SLOT, BOT_ID,
)

logger = telebot.logger
telebot.logger.setLevel(logging.INFO)

bot = AsyncTeleBot(BOT_TOKEN, state_storage=StateMemoryStorage())

ALL_PROJECTS = [
]

async def verify_file(verify_bin: str, proof: str, rpc_url: str):
    args = [
        verify_bin,
        "verify",
        "-r",
        rpc_url,
        str(proof)
    ]
    
    proc = await asyncio.create_subprocess_exec(
        *args,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    data = await proc.stdout.read()
    err = await proc.stderr.read()
    await proc.wait()
    if proc.returncode != 0:
        return False, err.decode()
    return True, json.loads(data)


welcome_message = """
Welcome to ODESEC bot!

You can send me a proof file and I will verify it for you.

Your contact info is: {contact}
"""

@bot.message_handler(commands=['start', 'help',])
async def send_welcome(message):
    # print("commands: ", message)
    contact = "tg:{}/{}".format(BOT_ID, message.chat.id)
    msg = welcome_message.replace("{contact}", str(contact))
    bot.reply_to(message, msg)



class SubmitStates(StatesGroup):
    description = State()


@bot.message_handler(content_types=['document'])
async def handle_file(message: telebot.types.Message):
    print("docs message", message)
    if not message.document.file_name.endswith(".zkp"):
        await bot.reply_to(message, "Only .zkp files are allowed")
        return
    caption = (message.caption or '').strip()
    if not caption:
        await bot.reply_to(message, "Please send me a caption with contact info")
        return
    
    project = None
    for row in ALL_PROJECTS:
        if row["contact"] == caption:
            project = row
            break
    
    if not project:
        await bot.reply_to(message, "Project not found")
        return
    result_message = await bot.send_message(message.chat.id, "<i>Download your proof...</i>", parse_mode='HTML', disable_web_page_preview=True)
    file_path = await bot.get_file(message.document.file_id)
    download_file = await bot.download_file(file_path.file_path)
    temp_dir = tempfile.TemporaryDirectory()
    
    save_file_path = os.path.join(temp_dir.name, "odsec.zkp")
    with open(save_file_path, 'wb') as new_file:
        new_file.write(download_file)
    
    await bot.edit_message_text(chat_id=message.chat.id, message_id=result_message.message_id, text="<i>Verifying your proof...</i>", parse_mode='HTML')
    # bot.reply_to(message, "Document received")
    logger.info("receiv a new proof from user %s, name: %s", message.from_user.username, message.document.file_name)

    (is_valid, data) = await verify_file(ODESEC_BIN, save_file_path, FULL_NODE_RPC_URL)
    if is_valid:
        # check the POC is profitable
        is_profitable = False
        if DEFAULT_POC_ADDRESS.lower() in data:
            account_data = data[DEFAULT_POC_ADDRESS.lower()]
            if account_data.get("balance"):
                is_profitable = True
        
        for token_address, slot in FAMOUS_BALANCE_SLOT:
            if token_address.lower() in data:
                token_data = data[token_address.lower()]
                if slot in (token_data.get('storage') or {}):
                    is_profitable = True
        if is_profitable:
            # check the contract address of the project is afftected
            project_address = set(x.lower() for x in project["contracts"])
            affetcted_address = set(x.lower() for x in data.keys())
            is_affected = bool(project_address.intersection(affetcted_address))

            if not is_affected:
                logger.info("project is not affected")
                is_valid = False
        else:
            logger.info("poc is not profitable")
            is_valid = False
    else:
        logger.info("zkp is invalid: %s", data)
    if not is_valid:
        await bot.edit_message_text(chat_id=message.chat.id, message_id=result_message.message_id, text="<i>Proof is invalid!</i>", parse_mode='HTML')
        return

    await bot.edit_message_text(chat_id=message.chat.id, message_id=result_message.message_id, text="<i>Proof is valid!</i>", parse_mode='HTML')
    
    await bot.set_state(message.from_user.id, SubmitStates.description, message.chat.id)
    async with bot.retrieve_data(message.from_user.id, message.chat.id) as data:
        data["to"] = project

    await bot.send_message(message.chat.id, "Please make a short description of the emergency.")
    # async with bot.retrieve_data(message.from_user.id, message.chat.id) as data:

    # bot.send_message("-4138107020", "There is a ")
    # else:
    #     print("zkp is invalid: ", data)
    #     await bot.edit_message_text(chat_id=message.chat.id, message_id=result_message.message_id, text="<i>Proof is invalid!</i>", parse_mode='HTML')
    # bot.send_message("-4138107020", "Document received")

@bot.message_handler(state=SubmitStates.description)
async def handle_description(message: telebot.types.Message):
    async with bot.retrieve_data(message.from_user.id, message.chat.id) as data:
        chat_id = data["to"]["contact"].rsplit('/')[-1]
        try:
            await bot.send_message(
                chat_id, 
                formatting.format_text(
                    f"New emergency from @{message.from_user.username}, ",
                    formatting.mcite(message.text)
                ),
                parse_mode='MarkdownV2'
            )
        except Exception as e:
            logger.warning("error: %r", e)
 
    await bot.delete_state(message.from_user.id, message.chat.id)
    await bot.send_message(chat_id=message.chat.id, text="Thank you for your submission!, The project team will contact you as soon as possible.")


@bot.message_handler(func=lambda message: True)
async def echo_all(message):
    print("message: ", message.chat.username, message.chat.id, message.text)
    # bot.reply_to(message, message.text)


ODESEC_ABI = json.loads(
    '[{"inputs":[{"internalType":"address","name":"_owner","type":"address"},{"internalType":"contract IRiscZeroVerifier","name":"_verifier","type":"address"},{"internalType":"bytes32","name":"_imageId","type":"bytes32"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":false,"internalType":"uint256","name":"projectId","type":"uint256"},{"indexed":false,"internalType":"string","name":"domain","type":"string"},{"indexed":false,"internalType":"address","name":"owner","type":"address"}],"name":"ProjectAdded","type":"event"},{"inputs":[],"name":"MAGIC","outputs":[{"internalType":"bytes","name":"","type":"bytes"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"string","name":"_domain","type":"string"},{"internalType":"string","name":"_contact","type":"string"},{"internalType":"address[]","name":"_contracts","type":"address[]"},{"internalType":"address","name":"_owner","type":"address"},{"internalType":"bytes","name":"receipt","type":"bytes"}],"name":"addProject","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"limit","type":"uint256"},{"internalType":"uint256","name":"offset","type":"uint256"}],"name":"getProjectList","outputs":[{"components":[{"internalType":"address","name":"owner","type":"address"},{"internalType":"address[]","name":"contracts","type":"address[]"},{"internalType":"string","name":"domain","type":"string"},{"internalType":"string","name":"contact","type":"string"}],"internalType":"struct ODESEC.ProjectData[]","name":"","type":"tuple[]"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"imageId","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"string","name":"domain","type":"string"},{"internalType":"address","name":"_owner","type":"address"}],"name":"makeChallenge","outputs":[{"internalType":"bytes20","name":"","type":"bytes20"}],"stateMutability":"pure","type":"function"},{"inputs":[],"name":"owner","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"string","name":"_domain","type":"string"}],"name":"projectIdOfDomain","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"name":"projectIds","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"uint256","name":"","type":"uint256"}],"name":"projects","outputs":[{"internalType":"address","name":"owner","type":"address"},{"internalType":"string","name":"domain","type":"string"},{"internalType":"string","name":"contact","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"totalProjects","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"bytes32","name":"_imageId","type":"bytes32"}],"name":"updateImageId","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"projectId","type":"uint256"},{"internalType":"string","name":"contact","type":"string"},{"internalType":"address[]","name":"contracts","type":"address[]"}],"name":"updateProject","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[],"name":"verifier","outputs":[{"internalType":"contract IRiscZeroVerifier","name":"","type":"address"}],"stateMutability":"view","type":"function"}]'
)

async def update_odesec_projects():
    global ALL_PROJECTS
    w3 = AsyncWeb3(AsyncHTTPProvider(FULL_NODE_RPC_URL))
    odesec_contract = w3.eth.contract(
        address=ODESEC_CONTRACT_ADDRESS,
        abi=ODESEC_ABI
    )
    total_projects = await odesec_contract.functions.totalProjects().call()
    if total_projects <= len(ALL_PROJECTS):
        return
    logger.info("update projects, total: %s", total_projects)
    new_projects = await odesec_contract.functions.getProjectList(10, len(ALL_PROJECTS)).call()
    for row in new_projects:
        item = {
            'owner': row[0],
            'contracts': row[1],
            'domain': row[2],
            'contact': row[3]
        }
        ALL_PROJECTS.append(item)
    

async def scheduler():
    while True:
        await aioschedule.run_pending()
        await asyncio.sleep(1)


bot.add_custom_filter(asyncio_filters.StateFilter(bot))
async def main():
    aioschedule.every(10).seconds.do(update_odesec_projects)
    await asyncio.gather(bot.infinity_polling(), scheduler())

if __name__ == '__main__':
    asyncio.run(main())
