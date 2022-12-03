

const telegramchatiddv = "telegramchatiddv"
const telegramchatidbot = "telegramchatidbot"
const deviceid = "deviceid"
const options = '{}'
const cryptopassword = "password"
const telegramtoken = "token"

const boxfolders = `dotfiles/local:dotfiles
Tasker
epsxe/memcards
/data/data/com.termux/files/home/.termux:Termux/termux`

const opts = JSON.parse(options || '{}')

function getTelegramChatId(opts) {
   const originalCase = opts?.telegram?.chat || 'bot'
   return originalCase.toLowerCase()
}

function shouldLog(opts) {
   const v = opts?.tasker?.log
   console.log({ v })
   return (v === undefined || v === null) ? true : v
}

const telegramChatIds = { 'dv': telegramchatidbot, 'bot': telegramchatidbot }
const telegramChatId = telegramChatIds[getTelegramChatId(opts)]

const taskerYaml = shouldLog(opts) ? `tasker: 
  log_task: "Log Storm progress"` : ""

const boxFoldersYaml = boxfolders.split('\n').map(l => {
   const [f, t] = l.split(':')
   const to = t || f
   const from = f.startsWith('/') ? f : `/sdcard/${f}`
   return `    - from: "${from}"
      to: "${to}"`
}).join('\n')

const fullYaml = `
backup:
  provider: box
  to: Devices/${deviceid}
  max_kb: 1024
  denylist:
    - ".app"
  paths:
${boxFoldersYaml}
cloud: 
  providers:
    alumni: 
      buffer: /sdcard/Storm/alumni
      rclone: alumni
    box: 
      buffer: /sdcard/Storm/box
      rclone: box
    gphotos: 
      buffer: /sdcard/DCIM/GPhotos
      single_folder: true
    vvgphotos:
      buffer: /sdcard/Storm/VVGPhotos
      rclone: vvgphotos
      single_folder: true
    pcloud: 
      buffer: /sdcard/Storm/pcloud
      rclone: pcloud
    telegram: 
      buffer: /sdcard/Storm/telegram
crypto: 
  password: ${cryptopassword}
camera: 
  paths:
    - from: /sdcard/Pictures/Camera
      to: Pictures/Camera
      low_unzipped: gphotos
    - from: /sdcard/Pictures/VV/Camera
      to: VV/Pictures/Camera
      low_unzipped: vvgphotos
db:
  path: /sdcard/Tasker/db/storm_storage.txt
telegram:
  token: ${telegramtoken}
  chat_id: ${telegramChatId}
${taskerYaml}
`

console.log(fullYaml)
// setLocal("config", fullYaml.replaceAll('"', '@'))