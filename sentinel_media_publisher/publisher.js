const fs = require('fs');
const { google } = require('googleapis');
const path = require('path');
const readline = require('readline');

// Configuración
const SCOPES = ['https://www.googleapis.com/auth/youtube.upload'];
const TOKEN_PATH = path.join(__dirname, 'token.json');
const CREDENTIALS_PATH = path.join(__dirname, '../client_secrets.json');

async function authorize(credentials) {
  const { client_secret, client_id, redirect_uris } = credentials.installed || credentials.web;
  const oAuth2Client = new google.auth.OAuth2(client_id, client_secret, redirect_uris[0]);

  try {
    const token = fs.readFileSync(TOKEN_PATH);
    oAuth2Client.setCredentials(JSON.parse(token));
    return oAuth2Client;
  } catch (err) {
    return getNewToken(oAuth2Client);
  }
}

function getNewToken(oAuth2Client) {
  return new Promise((resolve, reject) => {
    const authUrl = oAuth2Client.generateAuthUrl({ access_type: 'offline', scope: SCOPES });
    console.log('Authorize this app by visiting this url:', authUrl);
    const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
    rl.question('Enter the code from that page here: ', (code) => {
      rl.close();
      oAuth2Client.getToken(code, (err, token) => {
        if (err) return reject('Error retrieving access token', err);
        oAuth2Client.setCredentials(token);
        fs.writeFileSync(TOKEN_PATH, JSON.stringify(token));
        resolve(oAuth2Client);
      });
    });
  });
}

async function uploadVideo(auth, filePath, title, description, privacy) {
  const service = google.youtube('v3');
  const fileSize = fs.statSync(filePath).size;
  
  console.log(`🚀 Uploading: ${title} (${privacy})`);
  
  const res = await service.videos.insert({
    auth: auth,
    part: 'snippet,status',
    requestBody: {
      snippet: {
        title: title,
        description: description,
        tags: ['Sentinel', 'AI', 'NodeJS'],
        categoryId: '28',
      },
      status: {
        privacyStatus: privacy,
      },
    },
    media: {
      body: fs.createReadStream(filePath),
    },
  }, {
    onUploadProgress: evt => {
      const progress = (evt.bytesRead / fileSize) * 100;
      process.stdout.write(`\rProgress: ${Math.round(progress)}%`);
    },
  });
  
  console.log('\n✅ Upload complete!');
  console.log(`https://youtu.be/${res.data.id}`);
}

const args = process.argv.slice(2);
const fileArg = args.find(a => a.startsWith('--file='));
const titleArg = args.find(a => a.startsWith('--title='));
const descArg = args.find(a => a.startsWith('--desc='));
const privacyArg = args.find(a => a.startsWith('--privacy=')) || '--privacy=private';

if (!fileArg) {
  console.error("Usage: node publisher.js --file=VIDEO.mp4 [--title=TITLE] [--desc=DESC] [--privacy=private]");
  process.exit(1);
}

const filePath = fileArg.split('=')[1];
const title = titleArg ? titleArg.split('=')[1] : path.basename(filePath, path.extname(filePath));
const desc = descArg ? descArg.split('=')[1] : "Uploaded by Sentinel Node Publisher";
const privacy = privacyArg.split('=')[1];

fs.readFile(CREDENTIALS_PATH, (err, content) => {
  if (err) return console.log('Error loading client secret file:', err);
  authorize(JSON.parse(content))
    .then(auth => uploadVideo(auth, filePath, title, desc, privacy))
    .catch(console.error);
});
