# Title: 
SUM3API: Using Rust, ZeroMQ, and MetaQuotes Language (MQL5) API Combination to Extract, Communicate, and Externally Project Financial Data from MetaTrader 5 (MT5)

## Author
Albeos Rembrant

## Role/Organization
Independent Researcher

## Email
algorembrant@gmail.com

## Date
2026-01-26

# Abstract

The MetaTrader 5 (MT5), connected to your preferred exchange or broker, supports automated algorithmic trading via Expert Advisors (EAs) written in MetaQuotes Language (MQL5). Also, MetaQuotes LLC provides an official Python integration package; there are limited publicly documented methods for internally extracting and externally projecting MT5 financial data, other than that. To address this gap, this study implements a unique approach that bridges MQL5 and Rust via ZeroMQ publisher–subscriber & request-reply bindings. Nothing is compared in this study, and this benchmark-based methodology enables quantitative researchers, feature engineers, and algorithmic traders to develop trading systems leveraging MT5 datafeeds using Rust, bypassing the limitations of pure MQL5 EAs. The methodology is proven through integrating it in one software application (simple trading termminal) demonstrating these low-latency functionality: includes real-time account info (balance, equity, free and used margin), historical data request (OCHL, raw tick), forward data request (live recorded), trade controls (buy and sell either Market, Limit, or Stop orders) with lotsizing and trade cancelation, messenging and notifications (debugging and recent calls), and a live microsecond raw tick-level bid/ask formation chart. All resources are open-source and available on GitHub: https://github.com/algorembrant/SUM3API

# Introduction

## Background
Research studies and methods substantiate API-based data requests as a common starting point in practical implementations, particularly for enabling scalable feature engineering and quantitative analysis in algorithmic trading (1, 2, 3, 4, 5, 6, 7). Acquiring datasets to model their own way of trading, analyzing portfolios, parsing event sentiments, and decoding trading narratives. 

API-access-database Projects

Both historical and recent practices fetch trading-data from various platforms to develop trading strategies. These platforms with accessible APIs are not designed for trading purposes but mainly to analyze portfolios and trading trends, namely Yahoo Finance & Twitter’s data is used for Long-Short Term Memory neural network (LSTM) modeled time series analysis (8), Alpha Vantage for Naive Bayes machine learning algorithm for predicting stock prices (9), Finnhub is used as part of a large-scale Financial News and Stock Price Integration Dataset (FNSPID) model for time series financial analysis (10), Massive (was Polygon.io) to aggregate news articles (11), IEX Cloud to retrieve real-time stock data to predict stock prices (12), SimFin’s balance sheet data for machine learning fundamental analysis to predict stock prices (13), Quandl’s dataset as a part of alternative data and sentiment analysis (14). And consideration of the cryptocurrency side, namely CoinGecko’s databases (15), CryptoCompare’s datasets for real-time data architecture (16), BraveNewCoin’s daily prices datasets for analysis of the entry and exit dynamics of the cryptocurrency market (17), Twelve Data’s datasets for quantitative analysis for Stocks and Cryptocurrencies (18), Messari’s datasets for style investing (19), Glassnode’s dataset for Bitcoin price direction prediction using on-chain data and feature selection (20). Some of these platforms do offer forex-based datasets, which are also used for studies, such as from Yahoo Finance’s datasets for forecasting with feature-augmented multivariate LSTM models (21), 

And many more platforms with API access for trading datasets. Although it is widely used for trading analysis and machine learning modeling, those platforms do not offer a trade-exectable APIs. Scholars are studying to create strategies, but they are not mainly for live deployment purposes, which is why we argue that's the reason why there models may fail in live environment since their datasets itself for modeling/practing doesnt have trade-exucutable API. If we 

Financial Modeling without the aim to implement it on live environment sucks.

Trading with two completely different platforms, one connected with the dataset API and one with connected has trade-executin API, may cause mispriceing ot mis alignment of prices. It was forecast and perfectly fine to have a stoploss of 2R, but the broker had the ad ada dip a few ticks below what was in the chatting platform.

two arguments: first, They conduct model construction using datasets from platforms that don't have trade-executable API support, meaning they just model and never meant to make it implement on live. Second, Modeling using two completely different platforms, one for charting and one for trade execution, may cause a misprice alignment that may lead to unfortunate losses even if run in real-time. Was modeled perfectly, but failed with a few ticks on live.

The great example of this is TradingView same assestr 



by also implementing TN the model in live environment, yhen we must use platforms that offers both API for datasets and API for trade-exucution. For instance, Metatrader5 (MT5)





There are many scholarly studies besides what was mentioned above, with the use of platforms to access financial datasets to model their trading strategies, but the use of those platforms is almost irrelevant for ‘live-trading’, at least for lower timeframe trading (we are not specifically referring to HFTs trading). Its magnificent to design these machine learning based technologies but they often fail in live trading, simply because they analyze inside a dataset providers or chart platforms and if they place a trade from a completely different place, then there would be a mispricing alignment. Nnot only that, 






Unlike Binance, which has an externally accessible API for trade execution, brokers such as vlavlalvalvllavlalvalvlav doesnt duppoert that kind of API. But Meratrader addressthge issue. Metra trader has limited use as its native language was MWLr built after cc++, as of now at the time of writing, MT5 API is not that popular in scholarly studies for rkerhk, some claimed that it's due to old-like version bit havent, this study will ima a unique application of exploiting data. Less popular is the use of the MetaTrader5 Python library for developing trading studies.


Does not API-access-database Projects

Meanwhile, some brokers do not offer public external APIs for feature-modeling or algorithmic trading and are only accessible to private entities, namely Exness’s  API, IC Markets’s API, XM Group’s API, FPMarkets API’s, AvaTrade API’s, and more. Although APIs can't be accessed publicly and individually, there are ways to extract data, feature modeling, and conduct algorithmic trading. MetaTrader 5 (MT5) is a third-party trading platform (22) developed by MetaQuotes Software, which recently won Prestigious Awards at Forex Expo Dubai 2025 (23). It is designed to connect users to their chosen forex broker and offers advanced trading tools and features, making it suitable for multi-asset trading across various instruments. 

There is no problem with using platforms that offer an API to access financial datasets, which give data for modeling trading strategies. On the other hand,  if we were planning to elevate the modeling approach, then we would also consider making it compatible in a live environment. Simply by using one platform that offers both an API for dataset access and an API for trade-execution. One example of that is in Binance (24).  Some studies are dedicated to designing both financial modeling and live implementation using Binance APIs,  such as a pipeline using PostgreSQL for historical data storage, Redis for real-time caching of Binance WebSocket streams, and the Binance REST API for trade execution (25), a backtesting and portfolio optimization to live trading results on Binance Futures with a pipeline consists of universe selection, alpha backtesting, volatility aware portfolio optimization, and dynamic drawdown-based risk management (26), FinRL is an open-source deep reinforcement learning framework that acts as a full pipeline from strategy design to simulated trading and can interface with live trading APIs for execution (27), and more. Despite that, Binance does not offer traditional forex trading. Instead, it provides a crypto-oriented approach to forex through several instruments. Binance allows trading in stablecoins such as USDT, BUSD, and USDC, which are pegged to the US dollar (28).


## Problem Statement
The MetaTrader 5 (MT5), connected to your preferred exchange or broker, supports automated algorithmic trading via Expert Advisors (EAs) written in MetaQuotes Language (MQL5). Also, MetaQuotes LLC provides an official Python integration package; there are limited publicly documented methods for internally extracting and externally projecting MT5 financial data, other than that.

## Objectives
To address this gap, this study implements a unique approach that bridges MQL5 and Rust via ZeroMQ publisher–subscriber & request-reply bindings.

## Significance

(partial concept, about: this benchmark-based methodology enables quantitative researchers, feature engineers, and algorithmic traders to develop trading systems leveraging MT5 datafeeds using Rust, bypassing the limitations of pure MQL5 EAs. 

# Methodology

## Research Design

(partial concept, about: everything is revolves arround discussed here 'C:\Users\User\Desktop\VSCode\SUM3API\MQL5-Rustmt5-chart-Documentation.md )

## Implementation

(partial concept, about: The implementation is done inside a 7-year old Windows 10 Pro 64-bit laptop with 8GB RAM and Intel Core i5-7200U CPU @ 2.50GHz 2.71 GHz, NVIDIA GeForce 940MX, and 256GB SSD. )

The MetaTrader (MT5) platform is already connected to Exness Broker Standard Cent Live Account. 

# Results

(partial concept, about: the detailed software demonstation is in the github repository)

## Discussion

(partial concept, about: demonstrating these low-latency functionality: includes real-time account info (balance, equity, free and used margin), historical data request (OCHL, raw tick), forward data request (live recorded), trade controls (buy and sell either Market, Limit, or Stop orders) with lotsizing and trade cancelation, messenging and notifications (debugging and recent calls), and a live microsecond raw tick-level bid/ask formation chart. All resources are open-source and available on GitHub: https://github.com/algorembrant/SUM3API)

# Conclusion

(partial concept, about: its really possible to make a unique approach of extracting and projecting financial data from MetaTrader 5 (MT5) using Rust, ZeroMQ, and MetaQuotes Language (MQL5) API combination. )

# References

[1] David Jukl, Jan Lansky. (2025). Systematic Review on Algorithmic Trading. Acta Informatica Pragensia, 14(3), 506-534. https://doi.org/10.18267/j.aip.276

[2] Jifeng Li, Arnav Grover, Abraham Alpuerto, Yupeng Cao, Xiao-Yang Liu. (2025). Orchestration Framework for Financial Agents: From Algorithmic Trading to Agentic Trading. https://arxiv.org/html/2512.02227v1

[3] Krishnamurthy Nayak, Supreetha Balavalikar Shivaram, Sumukha K. Nayak. (2025). Machine Learning Framework for Algorithmic Trading. Comput. Sci. Math. Forum 2025, 12(1), 12; https://doi.org/10.3390/cmsf2025012012

[4] Mohit Tyagi, Nookala Venu. (2025). An RSI-Based Algorithmic Trading System Using Angel One Smart API: Design, Implementation, and Performance Evaluation. https://www.ijfmr.com/papers/2025/3/45227.pdf

[5] Adam Darmanin, Vince Vella. (2025). Language Model Guided Reinforcement Learning in Quantitative Trading. https://arxiv.org/html/2508.02366v2

[6] Keyi Wang, Nikolaus Holzer, Ziyi Xia, Yupeng Cao, Jiechao Gao, Anwar Walid, Kairong Xiao, Xiao-Yang Liu, Yanglet. (2023). FinRL Contests: Benchmarking Data-driven Financial Reinforcement Learning Agents. https://arxiv.org/pdf/2504.02281v3

[7] Wentao Zhang, Yilei Zhao, Chuqiao Zong. (2025). FinWorld: An All-in-One Open-Source Platform for End-to-End Financial AI Research and Deployment. https://arxiv.org/html/2508.02292v1

[8] Baku, Azerbaijan. (2023). Predicting Financial Market Trends using Time Series Analysis and Natural Language Processing. https://arxiv.org/abs/2309.00136 

[9] Kunal Raut, Pinak Kasture, Chetan Gosavi, Tanmay Deshpande. (2022). Stock Market Prediction using Alpha Vantage API and Machine Learning Algorithm. https://www.irjet.net/archives/V9/i5/IRJET-V9I5162.pdf

[10] Zihan Dong, Xinyu Fan. (2024). FNSPID: A Comprehensive Financial News Dataset in Time Series. https://arxiv.org/html/2402.06698v1

[11] Marian Pompiliu Cristescu , Dumitru Alexandru Mara,* , Raluca Andreea Nerișanu , Lia Cornelia Culda and Ionela Maniu. (2023). Analyzing the Impact of Financial News Sentiments on Stock Prices—A Wavelet Correlation. https://doi.org/10.3390/math11234830

[12] Anurag Gupta, Dr. Vishal Shrivastava, Dr. Akhil Pandey, Er. Ram Babu Buri. (2025). Real-Time Stock Market Data Analysis and Visualization Using Power BI. https://ijrpr.com/uploads/V6ISSUE5/IJRPR45525.pdf

[13] AkshayKhanapuri, NarayanaDarapaneni, AnweshReddyPaduri. (2024). Utilizing Fundamental Analysis to Predict Stock Prices. https://doi.org/10.4108/airo.5140

[14] Kristian Bondo Hansen, Christian Borch. (2022). Alternative data and sentiment analysis: Prospecting non-standard data in machine learning-driven finance. https://doi.org/10.1177/20539517211070701

[15] David Vidal-Tomás. (2022) Which cryptocurrency data sources should scholars use? https://doi.org/10.1016/j.irfa.2022.102061

[16] Adrian Barradas, Acela Tejeda-Gil, Rosa-María Cantón-Croda. (2022) Real-Time Big Data Architecture for Processing Cryptocurrency and Social Media Data: A Clustering Approach Based on k-Means. https://doi.org/10.3390/a15050140

[17] David Vidal-Tomás. (2021).The entry and exit dynamics of the cryptocurrency market. https://doi.org/10.1016/j.ribaf.2021.101504

[18] Akshat Sharma; Ashtha Goyal; Durgaprasad Gangodkar; Yogesh Lohumi. Quantitative Analysis for Stocks and Cryptocurrencies using Python. (2024) https://doi.org/10.1109/ICEECT61758.2024.10739272

[19] Fatima Abd Rabbo, Mustafa Disli.  (2025). Style investing and return comovement in the cryptocurrency market. https://doi.org/10.1016/j.ribaf.2025.102949

[20] Ritwik Dubey, David Enke. (2025) Bitcoin price direction prediction using on-chain data and feature selection. https://doi.org/10.1016/j.mlwa.2025.100674

[21] Enhancing forex market forecasting with feature-augmented multivariate LSTM models using real-time data. (2025) https://doi.org/10.1016/j.knosys.2025.114500

[22] MetaTrader5. (2026). A powerful platform for Forex and Exchange markets. https://www.metatrader5.com/

[23] MetaQuotes LLC. (2025). MetaQuotes Wins Two Prestigious Awards at Forex Expo Dubai 2025. https://www.metaquotes.net/en/company/news/5498

[24] Binance. (2026). Binance Spot API Docs. GitHub. https://github.com/binance/binance-spot-api-docs

[25] Elisa Beraudo, Yurii Oliinyk. (2024). The automatic cryptocurrency trading system uses a scalping strategy. https://doi.org/10.20535/2786-8729.5.2024.316563

[26] Thanh Nguyen. (2025). Talyxion: From Speculation to Optimization in Risk-Managed Crypto Portfolio Allocation. https://doi.org/10.48550/arXiv.2511.13239

[27] Xiao-Yang Liu, Hongyang Yang, Jiechao Gao, Christina Dan Wang. (2022). FinRL: a deep reinforcement learning framework to automate trading in quantitative finance.
https://doi.org/10.1145/3490354.3494366

[28] BinanceUS. (2026). Listings on Binance.US | Supported crypto, networks, and trading pairs. https://support.binance.us/en/articles/9842915-listings-on-binance-us-supported-crypto-networks-and-trading-pairs




# Appendices
