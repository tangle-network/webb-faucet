const { createProxyMiddleware } = require("http-proxy-middleware");

module.exports = function(app) {
  app.use(
    "/2/oauth2/token",
    createProxyMiddleware({
      target: "https://api.twitter.com",
      changeOrigin: true,
      secure: false,
      preserveHeaderKeyCase: true
    })
  );
};
