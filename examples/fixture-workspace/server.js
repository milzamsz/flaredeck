const http = require("node:http");

http.createServer((request, response) => {
  response.writeHead(request.url === "/health" ? 204 : 200, { "content-type": "text/plain" });
  response.end(request.url === "/health" ? "" : "flaredeck fixture\n");
}).listen(3210, "127.0.0.1");
