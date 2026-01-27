Rembrant Oyangoren Albeos
Independent Researcher
ORCID: https://orcid.org/0009-0006-8743-4419
Email: algorembrant@gmail.com
GitHub: https://github.com/algorembrant/SUM3API
First version: Jan 20, this version Jan 27 2026



The implementation is done inside a 7-year old Windows 10 Pro 64-bit laptop with 8GB RAM and Intel Core i5-7200U CPU @ 2.50GHz 2.71 GHz, NVIDIA GeForce 940MX, and 256GB SSD

All metadata are fetched automatically, independent of the ticker or symbol attached to the files. For this implementation we could attach ZmqPublisher.mq5 to any chart (XAUUSDc or BTCUSDc, etc) without modifying the code. In context, we are using Exness broker with a Standard-cent live account for this study, not demo, not paper.


appendix 1. a comment superscript - I’m using Exness and a Standard-cent live trading account. Since I’m using a cent-based account, all tradable assets will now have a ‘c’ suffix (eg., XAUUSDc for gold, and BTCUSDc for Bitcoin, etc.).

appendix 1. a comment superscript - In my case, I’ve already installed applications such as Git, Rust, VisualStudio C++ Build tools, MetaTrader5, and MetaEditor5. Overall, I'm working on Google Antigravity (you may use any code editor) as long as we have a terminal. Any version would do.


appendix 1, a comment superscript - We havent compared any bid/ask formation (dataframes, linechart formation) to any other approach. Hence, we cant say we lost data or succesfully extract with no failure. By far microscrecond resolution raw-bid/ask formation (the x-axis is not fixed-time-inteval, the formation depends raw up-&-down tick movement count). Thus we only say Accurate tick datatransmission with no message loss "under normal market conditions"


appendix 1, a comment superscript - i have didecated python project on this, where all tradable assest (forex, 1 cypto) in Exness standard-cent live account. Github link: https://github.com/algorembrant/QRAT2025/blob/main/Python%20Folder/Ticker%20Quotes%20Fetching/Quotes%20of%20all%20tradable%20assets%20in%20Exness.ipynb


appendix 1, a comment superscript - for ( Single Symbol Scope: The current implementation operates on the chart symbol where the EA is attached; multisymbol support would require architecture extensions.) there is a way and ive done it based on my previous projects, i attached the EA on any symbol and it trades different symbols. There is a way to address this but i keep this research study as simple as possible. The code for that is not publicly available but you can contact me. The reason its private its because its EA (Automated algorithmic tradinc bot, a diversification model. My simple reseach documentation about that EA found at link: https://github.com/algorembrant/QRAT2025/blob/main/Research%20Papers/The%203HTMOSDSDT.rp2%20%3A%20Portfolio%20Diversification%20for%20%E2%80%99Three-hour%20Timeframe%20Market%20Order%20Strategy%20with%20Defined%20Stoploss%20and%20Dynamic%20Takeprofit%E2%80%99%20Trading%20Model/3HTMOSDSDT_rp2.pdf )


appendix 1, a comment superscript - not a complete library like MQL's python library but all tackled on SUM3API system


appendix 1, a comment superscript - the software loses live data if closed, the database only store 'requested' information and not auto hence we cant recover the data if lost connected (unless we toggle ON recording all the time)

appendix 1, a comment superscript - ive been in software development since mid November5, i haven't learned docker yet.

appendix 1, a comment superscript - The first version of this was discovered at January 20, 2026. At the time of writing, it's really hard to find benchmark research papers since no one is interested in binding MT5’s API using a ZMQ bridge to other programming languages such as Rust, Go, C++, and Python (creating a combination of 3). We search across the internet using 9 AI leading platforms, namely Grok, Gemini, Claude, ChatGPT, MSCopilot, Perplexity, Baidu (Ernie Bot), DeepSeek, and Qwen (1, 2, 3, 4, 5, 6, 7, 8, 9). Some of the AI’s responses end up hallucinating, suggesting a combination of 2, and not the strictly 3, but later been clarified and confirmed that there are no public projects yet to implement before this. We also searched across four scholarly libraries, such as Arxiv, Google Scholar, SSRN, and Baidu 学术 (10, 11, 12, 13). The lack of literature suggests that this study’s approach (specifically using MT5’s MQL5 API to fetch live-trading datafeed from its connected Broker) is bound to ZMQ, bringing the other programming languages are not that popular and not well explored. (insert links to 1-13). Detailed ZMQ (The DLLs require that you have the latest Visual C++ runtime (2015).) binding to MQL - https://github.com/dingmaotu/mql-zmq (14) ZMQ binding to Rust - https://github.com/zeromq/zmq.rs (15) ZMQ binding between MQL and JavaScript - https://github.com/EricSchles/bindings-mql4-5 (16). (add year date to that) But never the 3 API combination (MQL5, ZMQ, RUST). 





Acknowledgements {

I am an independent researcher (non-affiliated with professionally known institutions), developing this study from scratch to final distribution required a lot of time, but with the help of xAI’s Grok for searching for facts, and im able to finished this by instructing & being open to feedback from Google’s Gemini Pro, Anthropic’s ClaudeOpus4.5, inside the Google’s Google Antigravity. I thank all the mentioned companies; if it weren't for their masterpiece, I would have a very hard time completing this work. Thank you, always.

I acknowledge the ZeroMQ community for providing the robust messaging library that forms the foundation of this work, and the Rust community for the excellent async ecosystem enabling this implementation. If it weren't for their existence, I would not have an idea of how to connect these three APIs.

I thank myself for having a goal to contribute to the research community rather than presenting a non-implementable approach or showcase complicated methods, for answering one gap, for staying strong despite the unfamiliar challenges, and for willingness to explore and grow again after backing down.

}

