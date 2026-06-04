const WebSocket = require('ws');
const timeService = require('./time-service');

let broadcastInterval = null;

function handleConnection(wss) {
  // 广播模式：每2秒主动推送时间给所有客户端
  broadcastInterval = setInterval(() => {
    if (wss.clients.size === 0) return;

    const timeMsg = JSON.stringify({
      type: 'time',
      serverTime: timeService.getServerTimeMs(),
      t2: Date.now()
    });

    wss.clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(timeMsg);
      }
    });
  }, 2000);

  wss.on('connection', (ws) => {
    // 新客户端连接时，立即发送一次当前时间
    ws.send(JSON.stringify({
      type: 'time',
      serverTime: timeService.getServerTimeMs(),
      t2: Date.now()
    }));

    // 处理客户端的 getTime 请求（用于 RTT 测量）
    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data);
        if (msg.type === 'getTime') {
          ws.send(JSON.stringify({
            type: 'timeResponse',
            t1: msg.t1,
            serverTime: timeService.getServerTimeMs()
          }));
        }
      } catch (e) {}
    });

    ws.on('error', () => {});
  });

  wss.on('close', () => {
    if (broadcastInterval) {
      clearInterval(broadcastInterval);
    }
  });
}

module.exports = { handleConnection };
