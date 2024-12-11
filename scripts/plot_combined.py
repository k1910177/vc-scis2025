import os
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import matplotlib.ticker as ticker

v_widths = [256, 1024, 4096]
m_widths = [2, 4, 8]
powers = (np.arange(17)+1).tolist()

fig, ax1 = plt.subplots(figsize=(8, 6))
ax2 = ax1.twinx()

gas_price = 10 # in gwei
eth_price = 3500 # eth/usd

v_csv = pd.read_csv('result/verkle_verify.csv')
m_csv = pd.read_csv('result/merkle_verify.csv')

for v_width in v_widths:
    v_table = v_csv[(v_csv["width"] == v_width) & (v_csv["power"] >= 3) & (v_csv["power"] <= 17)]

    size = v_table["size"].to_numpy(dtype='int')
    v_gas = v_table["gas"].to_numpy()
    v_usd = v_gas * gas_price * eth_price * pow(10, -9);

    ax1.plot(size, v_gas, linestyle="solid", marker='o', label="Verkle Tree (k: " + str(v_width) + ")")
    ax2.plot(size, v_usd, linestyle="None")

for m_width in m_widths:
    m_table = m_csv[(m_csv["width"] == m_width) & (m_csv["power"] >= 3) & (m_csv["power"] <= 17)]

    size = m_table["size"].to_numpy(dtype='int')
    m_gas = m_table["gas"].to_numpy()
    m_usd = m_gas * gas_price * eth_price * pow(10, -9);

    ax1.plot(size, m_gas, linestyle="solid", marker='o', label="Merkle Tree (k: " + str(m_width) + ")")
    ax2.plot(size, m_usd, linestyle="None")

ax1.set_xscale('log')

ax1.set_ylim(ymin=0,ymax=750_000)
ax2.set_ylim(ymin=0,ymax=26)
ax1.ticklabel_format(axis='y', scilimits=[0, 0])
ax1.yaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f'{x / 1e6:.2f}'))
ax1.yaxis.set_major_locator(ticker.MultipleLocator(100000))
ax2.yaxis.set_major_locator(ticker.MultipleLocator(2))
ax1.xaxis.set_major_locator(ticker.LogLocator(base=10.0, numticks=10))
ax1.xaxis.set_minor_locator(ticker.LogLocator(base=10.0, numticks=17))
ax1.xaxis.set_tick_params(which='minor', labelbottom=False) 
plt.text(0, 1.04, '1e6', ha='left', va='top', transform=ax1.transAxes)

ax1.set_xlabel('Vector length (Element size = 32 bytes)')
ax1.set_ylabel('Gas usage')
ax2.set_ylabel('Transaction fee (USD)')
ax1.set_title('Verification Cost of Merkle Trees and Verkle Trees')
ax1.legend(facecolor='white')

plt.show()
if not os.path.exists('./result/figures'):
    os.makedirs('./result/figures')
fig.savefig("./result/figures/verify_combined.pdf",  transparent=True)
