//+------------------------------------------------------------------+
//|                                                 ZmqPublisher.mq5 |
//|                                    Copyright 2024, Antigravity   |
//|                                                                  |
//+------------------------------------------------------------------+
#property copyright "Antigravity"
#property link      ""
#property version   "2.00"

// Include our ZMQ wrapper
#include <Zmq/Zmq.mqh>

// Include trading functions
#include <Trade/Trade.mqh>

// Input parameters
input string InpPubAddress = "tcp://0.0.0.0:5555"; // Tick Publisher Address
input string InpRepAddress = "tcp://0.0.0.0:5556"; // Order Handler Address
input double InpDefaultSlippage = 10;              // Default Slippage (points)

CZmq *g_publisher;  // PUB socket for tick data
CZmq *g_responder;  // REP socket for order handling
CTrade g_trade;     // Trading helper

//+------------------------------------------------------------------+
//| Expert initialization function                                   |
//+------------------------------------------------------------------+
int OnInit()
  {
   Print("Initializing ZmqPublisher v2.0 with Order Support...");
   
   // Initialize tick publisher (PUB socket)
   g_publisher = new CZmq();
   if(!g_publisher.Init(ZMQ_PUB)) {
      Print("Failed to initialize ZMQ Publisher");
      return(INIT_FAILED);
   }
   if(!g_publisher.Bind(InpPubAddress)) {
      Print("Failed to bind publisher to ", InpPubAddress);
      return(INIT_FAILED);
   }
   Print("Tick Publisher bound to ", InpPubAddress);
   
   // Initialize order responder (REP socket)
   g_responder = new CZmq();
   if(!g_responder.Init(ZMQ_REP)) {
      Print("Failed to initialize ZMQ Responder");
      return(INIT_FAILED);
   }
   if(!g_responder.Bind(InpRepAddress)) {
      Print("Failed to bind responder to ", InpRepAddress);
      return(INIT_FAILED);
   }
   Print("Order Responder bound to ", InpRepAddress);
   
   // Configure trade settings
   g_trade.SetDeviationInPoints((ulong)InpDefaultSlippage);
   g_trade.SetTypeFilling(ORDER_FILLING_IOC);
   
   return(INIT_SUCCEEDED);
  }

//+------------------------------------------------------------------+
//| Expert deinitialization function                                 |
//+------------------------------------------------------------------+
void OnDeinit(const int reason)
  {
   Print("Deinitializing ZmqPublisher...");
   if(g_publisher != NULL) {
      g_publisher.Shutdown();
      delete g_publisher;
      g_publisher = NULL;
   }
   if(g_responder != NULL) {
      g_responder.Shutdown();
      delete g_responder;
      g_responder = NULL;
   }
  }

//+------------------------------------------------------------------+
//| Process incoming order request                                   |
//+------------------------------------------------------------------+
//+------------------------------------------------------------------+
//| Process incoming order request                                   |
//+------------------------------------------------------------------+
string ProcessOrderRequest(string request)
  {
   // Expected JSON format:
   // {"type":"market_buy"|"close_position"|"cancel_order"|..., 
   //  "symbol":"XAUUSD", "volume":0.01, "price":2000.0, "ticket":12345}
   
   string orderType = ExtractJsonString(request, "type");
   string symbol = ExtractJsonString(request, "symbol");
   double volume = ExtractJsonDouble(request, "volume");
   double price = ExtractJsonDouble(request, "price");
   ulong ticket = (ulong)ExtractJsonDouble(request, "ticket"); // Simple extraction
   
   if(symbol == "") symbol = _Symbol;
   if(volume <= 0) volume = 0.01;
   
   Print("Order request: type=", orderType, " symbol=", symbol, " vol=", volume, " price=", price, " ticket=", ticket);
   
   bool success = false;
   ulong resultTicket = 0;
   string errorMsg = "";
   
   // Execute order based on type
   if(orderType == "market_buy") {
      double askPrice = SymbolInfoDouble(symbol, SYMBOL_ASK);
      success = g_trade.Buy(volume, symbol, askPrice, 0, 0, "Rust GUI Order");
      if(success) resultTicket = g_trade.ResultOrder();
      else errorMsg = GetLastErrorDescription();
   }
   else if(orderType == "market_sell") {
      double bidPrice = SymbolInfoDouble(symbol, SYMBOL_BID);
      success = g_trade.Sell(volume, symbol, bidPrice, 0, 0, "Rust GUI Order");
      if(success) resultTicket = g_trade.ResultOrder();
      else errorMsg = GetLastErrorDescription();
   }
   else if(orderType == "limit_buy") {
      success = g_trade.BuyLimit(volume, price, symbol, 0, 0, ORDER_TIME_GTC, 0, "Rust GUI Limit");
      if(success) resultTicket = g_trade.ResultOrder();
      else errorMsg = GetLastErrorDescription();
   }
   else if(orderType == "limit_sell") {
      success = g_trade.SellLimit(volume, price, symbol, 0, 0, ORDER_TIME_GTC, 0, "Rust GUI Limit");
      if(success) resultTicket = g_trade.ResultOrder();
      else errorMsg = GetLastErrorDescription();
   }
   else if(orderType == "stop_buy") {
      success = g_trade.BuyStop(volume, price, symbol, 0, 0, ORDER_TIME_GTC, 0, "Rust GUI Stop");
      if(success) resultTicket = g_trade.ResultOrder();
      else errorMsg = GetLastErrorDescription();
   }
   else if(orderType == "stop_sell") {
      success = g_trade.SellStop(volume, price, symbol, 0, 0, ORDER_TIME_GTC, 0, "Rust GUI Stop");
      if(success) resultTicket = g_trade.ResultOrder();
      else errorMsg = GetLastErrorDescription();
   }
   else if(orderType == "close_position") {
      if(ticket > 0) {
         success = g_trade.PositionClose(ticket);
         if(success) errorMsg = "Position closed";
         else errorMsg = GetLastErrorDescription();
      } else {
         errorMsg = "Invalid ticket for close_position";
      }
   }
   else if(orderType == "cancel_order") {
      if(ticket > 0) {
         success = g_trade.OrderDelete(ticket);
         if(success) errorMsg = "Order deleted";
         else errorMsg = GetLastErrorDescription();
      } else {
         errorMsg = "Invalid ticket for cancel_order";
      }
   }
   else if(orderType == "download_history") {
       // Format: {type: "download_history", symbol: "XAUUSD", timeframe: "M1", start: "2024.01.01", end: "2024.01.02", mode: "OHLC"|"TICKS"}
       string tfStr = ExtractJsonString(request, "timeframe");
       string startStr = ExtractJsonString(request, "start");
       string endStr = ExtractJsonString(request, "end");
       string mode = ExtractJsonString(request, "mode");
       
       if(mode == "") mode = "OHLC";
       
       success = DownloadHistory(symbol, tfStr, startStr, endStr, mode, errorMsg);
   }
   else {
      errorMsg = "Unknown order type: " + orderType;
   }
   
   // Build response JSON
   string response;
   if(success) {
      if(orderType == "download_history") {
          // ensure errorMsg contains the filename if success
          StringConcatenate(response, "{\"success\":true,\"message\":\"", errorMsg, "\"}");
      } else {
          StringConcatenate(response, "{\"success\":true,\"ticket\":", IntegerToString(resultTicket), "}");
      }
   } else {
      StringConcatenate(response, "{\"success\":false,\"error\":\"", errorMsg, "\"}");
   }
   
   return response;
  }

//+------------------------------------------------------------------+
//| Download History - Returns CSV content via ZMQ                   |
//+------------------------------------------------------------------+
bool DownloadHistory(string symbol, string tfStr, string startStr, string endStr, string mode, string &resultMsg)
  {
   datetime start = StringToTime(startStr);
   datetime end = StringToTime(endStr);
   if(start == 0) start = D'2024.01.01 00:00'; // Default fallback
   if(end == 0) end = TimeCurrent();
   
   ENUM_TIMEFRAMES tf = PERIOD_M1;
   if(tfStr == "M5") tf = PERIOD_M5;
   else if(tfStr == "M15") tf = PERIOD_M15;
   else if(tfStr == "H1") tf = PERIOD_H1;
   else if(tfStr == "H4") tf = PERIOD_H4;
   else if(tfStr == "D1") tf = PERIOD_D1;
   
   string csvContent = "";
   int count = 0;
   
   if(mode == "TICKS") {
      MqlTick ticks[];
      int received = CopyTicksRange(symbol, ticks, COPY_TICKS_ALL, start * 1000, end * 1000);
      
      if(received > 0) {
         csvContent = "Time,Bid,Ask,Last,Volume,Flags\n";
         for(int i=0; i<received && i<50000; i++) {  // Limit to 50k rows for ZMQ
            csvContent += TimeToString(ticks[i].time, TIME_DATE|TIME_SECONDS) + "," +
                         DoubleToString(ticks[i].bid, _Digits) + "," +
                         DoubleToString(ticks[i].ask, _Digits) + "," +
                         DoubleToString(ticks[i].last, _Digits) + "," +
                         IntegerToString(ticks[i].volume) + "," +
                         IntegerToString(ticks[i].flags) + "\n";
         }
         count = MathMin(received, 50000);
      }
   } 
   else {
      // OHLC
      MqlRates rates[];
      ArraySetAsSeries(rates, false);
      int received = CopyRates(symbol, tf, start, end, rates);
      
      if(received > 0) {
         csvContent = "Time,Open,High,Low,Close,TickVol,Spread,RealVol\n";
         for(int i=0; i<received && i<100000; i++) {  // Limit to 100k rows for OHLC
            csvContent += TimeToString(rates[i].time, TIME_DATE|TIME_MINUTES) + "," +
                         DoubleToString(rates[i].open, _Digits) + "," +
                         DoubleToString(rates[i].high, _Digits) + "," +
                         DoubleToString(rates[i].low, _Digits) + "," +
                         DoubleToString(rates[i].close, _Digits) + "," +
                         IntegerToString(rates[i].tick_volume) + "," +
                         IntegerToString(rates[i].spread) + "," +
                         IntegerToString(rates[i].real_volume) + "\n";
         }
         count = MathMin(received, 100000);
      }
   }
   
   if(count > 0) {
      // Return CSV content in a special format that Rust can parse
      // We use ||CSV_DATA|| as delimiter to separate count info from actual data
      resultMsg = IntegerToString(count) + " records||CSV_DATA||" + csvContent;
      return true;
   } else {
      resultMsg = "No data found for period";
      return false;
   }
  }

//+------------------------------------------------------------------+
//| Extract string value from JSON                                   |
//+------------------------------------------------------------------+
string ExtractJsonString(string json, string key)
  {
   string searchKey = "\"" + key + "\":\"";
   int startPos = StringFind(json, searchKey);
   if(startPos < 0) return "";
   
   startPos += StringLen(searchKey);
   int endPos = StringFind(json, "\"", startPos);
   if(endPos < 0) return "";
   
   return StringSubstr(json, startPos, endPos - startPos);
  }

//+------------------------------------------------------------------+
//| Extract double value from JSON                                   |
//+------------------------------------------------------------------+
double ExtractJsonDouble(string json, string key)
  {
   string searchKey = "\"" + key + "\":";
   int startPos = StringFind(json, searchKey);
   if(startPos < 0) return 0.0;
   
   startPos += StringLen(searchKey);
   
   // Find end of number (comma, }, or end of string)
   int endPos = startPos;
   int len = StringLen(json);
   while(endPos < len) {
      ushort ch = StringGetCharacter(json, endPos);
      if(ch == ',' || ch == '}' || ch == ' ') break;
      endPos++;
   }
   
   string valueStr = StringSubstr(json, startPos, endPos - startPos);
   return StringToDouble(valueStr);
  }

//+------------------------------------------------------------------+
//| Get human-readable error description                             |
//+------------------------------------------------------------------+
string GetLastErrorDescription()
  {
   int err = GetLastError();
   return "Error " + IntegerToString(err) + ": " + ErrorDescription(err);
  }

//+------------------------------------------------------------------+
//| Error description helper                                         |
//+------------------------------------------------------------------+
string ErrorDescription(int error)
  {
   switch(error) {
      case 0: return "No error";
      case 10004: return "Requote";
      case 10006: return "Request rejected";
      case 10007: return "Request canceled by trader";
      case 10010: return "Request rejected - only part of the request was fulfilled";
      case 10011: return "Request error";
      case 10012: return "Request canceled due to timeout";
      case 10013: return "Invalid request";
      case 10014: return "Invalid volume";
      case 10015: return "Invalid price";
      case 10016: return "Invalid stops";
      case 10017: return "Trade disabled";
      case 10018: return "Market is closed";
      case 10019: return "Not enough money";
      case 10020: return "Prices changed";
      case 10021: return "No quotes to process request";
      case 10022: return "Invalid order expiration date";
      case 10023: return "Order state changed";
      case 10024: return "Too many requests";
      case 10025: return "No changes in request";
      case 10026: return "Autotrading disabled by server";
      case 10027: return "Autotrading disabled by client terminal";
      case 10028: return "Request locked for processing";
      case 10029: return "Long positions only allowed";
      case 10030: return "Maximum position volume exceeded";
      default: return "Unknown error";
   }
  }

//+------------------------------------------------------------------+
//| Expert tick function                                             |
//+------------------------------------------------------------------+
void OnTick()
  {
   // Handle order requests (non-blocking)
   if(g_responder != NULL) {
      string request = g_responder.Receive(true);
      if(request != "") {
         Print("Received order request: ", request);
         string response = ProcessOrderRequest(request);
         g_responder.Send(response, false);  // Blocking send for REP pattern
         Print("Sent response: ", response);
      }
   }
   
   // Publish tick data with account info
   if(g_publisher == NULL) return;
   
   MqlTick tick;
   if(SymbolInfoTick(_Symbol, tick)) {
      // Get account info
      double balance = AccountInfoDouble(ACCOUNT_BALANCE);
      double equity = AccountInfoDouble(ACCOUNT_EQUITY);
      double margin = AccountInfoDouble(ACCOUNT_MARGIN);
      double freeMargin = AccountInfoDouble(ACCOUNT_MARGIN_FREE);
      
      // Get symbol trading constraints
      double minLot = SymbolInfoDouble(_Symbol, SYMBOL_VOLUME_MIN);
      double maxLot = SymbolInfoDouble(_Symbol, SYMBOL_VOLUME_MAX);
      double lotStep = SymbolInfoDouble(_Symbol, SYMBOL_VOLUME_STEP);
      
      // Get Active Positions (Only for current symbol to simplify)
      string positionsJson = "[";
      int posCount = PositionsTotal();
      bool firstPos = true;
      for(int i = 0; i < posCount; i++) {
         ulong ticket = PositionGetTicket(i);
         if(PositionSelectByTicket(ticket)) {
            if(PositionGetString(POSITION_SYMBOL) == _Symbol) {
               if(!firstPos) StringAdd(positionsJson, ",");
               
               string posType = (PositionGetInteger(POSITION_TYPE) == POSITION_TYPE_BUY) ? "BUY" : "SELL";
               StringAdd(positionsJson, "{\"ticket\":" + IntegerToString(ticket) + 
                         ",\"type\":\"" + posType + "\"" +
                         ",\"volume\":" + DoubleToString(PositionGetDouble(POSITION_VOLUME), 2) +
                         ",\"price\":" + DoubleToString(PositionGetDouble(POSITION_PRICE_OPEN), _Digits) +
                         ",\"profit\":" + DoubleToString(PositionGetDouble(POSITION_PROFIT), 2) + 
                         "}");
               firstPos = false;
            }
         }
      }
      StringAdd(positionsJson, "]");
      
      // Get Pending Orders (Only for current symbol)
      string ordersJson = "[";
      int orderCount = OrdersTotal();
      bool firstOrder = true;
      for(int i = 0; i < orderCount; i++) {
         ulong ticket = OrderGetTicket(i);
         if(OrderSelect(ticket)) {
            if(OrderGetString(ORDER_SYMBOL) == _Symbol) {
               if(!firstOrder) StringAdd(ordersJson, ",");
               
               ENUM_ORDER_TYPE type = (ENUM_ORDER_TYPE)OrderGetInteger(ORDER_TYPE);
               string orderTypeStr = "UNKNOWN";
               if(type == ORDER_TYPE_BUY_LIMIT) orderTypeStr = "BUY LIMIT";
               else if(type == ORDER_TYPE_SELL_LIMIT) orderTypeStr = "SELL LIMIT";
               else if(type == ORDER_TYPE_BUY_STOP) orderTypeStr = "BUY STOP";
               else if(type == ORDER_TYPE_SELL_STOP) orderTypeStr = "SELL STOP";
               
               StringAdd(ordersJson, "{\"ticket\":" + IntegerToString(ticket) + 
                         ",\"type\":\"" + orderTypeStr + "\"" +
                         ",\"volume\":" + DoubleToString(OrderGetDouble(ORDER_VOLUME_INITIAL), 2) +
                         ",\"price\":" + DoubleToString(OrderGetDouble(ORDER_PRICE_OPEN), _Digits) +
                         "}");
               firstOrder = false;
            }
         }
      }
      StringAdd(ordersJson, "]");
      
      // Create JSON with tick data + account info + positions + orders
      string json;
      StringConcatenate(json, "{\"symbol\":\"", _Symbol, 
                        "\",\"bid\":", DoubleToString(tick.bid, _Digits),
                        ",\"ask\":", DoubleToString(tick.ask, _Digits),
                        ",\"time\":", IntegerToString(tick.time),
                        ",\"volume\":", IntegerToString(tick.volume),
                        ",\"balance\":", DoubleToString(balance, 2),
                        ",\"equity\":", DoubleToString(equity, 2),
                        ",\"margin\":", DoubleToString(margin, 2),
                        ",\"free_margin\":", DoubleToString(freeMargin, 2),
                        ",\"min_lot\":", DoubleToString(minLot, 2),
                        ",\"max_lot\":", DoubleToString(maxLot, 2),
                        ",\"lot_step\":", DoubleToString(lotStep, 2),
                        ",\"positions\":", positionsJson,
                        ",\"orders\":", ordersJson,
                        "}");
                        
      g_publisher.Send(json);
      // Print("Published: ", json); // Uncomment for debugging (spammy)
   }
  }
  
//+------------------------------------------------------------------+
