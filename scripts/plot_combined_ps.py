import os
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import matplotlib.ticker as ticker

v_widths = [256, 1024, 4096]
m_widths = [2, 4, 8]
powers = (np.arange(17)+1).tolist()

fig_ps, ax = plt.subplots(figsize=(8,6))

v_csv = pd.read_csv('result/verkle_verify.csv')
m_csv = pd.read_csv('result/merkle_verify.csv')

for v_width in v_widths:
    v_table = v_csv[(v_csv["width"] == v_width) & (v_csv["power"] >= 3) & (v_csv["power"] <= 17)]

    size = v_table["size"].to_numpy(dtype='int')
    v_ps = v_table["proof_size"].to_numpy(dtype='int')

    ax.plot(size, v_ps, linestyle="solid", marker='o', label="Verkle Tree (k: " + str(v_width) + ")")

for m_width in m_widths:
    m_table = m_csv[(m_csv["width"] == m_width) & (m_csv["power"] >= 3) & (m_csv["power"] <= 17)]

    size = m_table["size"].to_numpy(dtype='int')

    m_ps = m_table["proof_size"].to_numpy(dtype='int')
    ax.plot(size, m_ps, linestyle="solid", marker='o', label="Merkle Tree (k: " + str(m_width) + ")")

ax.set_xscale('log')

ax.xaxis.set_major_locator(ticker.LogLocator(base=10.0, numticks=10))
ax.xaxis.set_minor_locator(ticker.LogLocator(base=10.0, numticks=17))
ax.xaxis.set_tick_params(which='minor', labelbottom=False) 

ax.set_xlabel('Vector length (Element size = 32 bytes)')
ax.set_ylabel('Proof Size (byte)')
ax.set_title('Proof Size of Merkle Trees and Verkle Trees')
ax.legend(facecolor='white')

plt.show()
if not os.path.exists('./result/figures'):
    os.makedirs('./result/figures')
fig_ps.savefig("./result/figures/verify_combined_ps.pdf",  transparent=True)
