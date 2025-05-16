import matplotlib.pyplot as plt
import pandas as pd
import sys

df = pd.read_csv(sys.argv[1], skipinitialspace=True)
df.columns = df.columns.str.strip()

df['fip'] = df['fip'].astype(bool)

# Plotting
for file_name in df['file'].unique():
    file_df = df[df['file'] == file_name]

    fig, axs = plt.subplots(1, 2, figsize=(12, 5))
    fig.suptitle(f"{file_name}", fontsize=14)

    # Subplot 1: Bar chart for max_mem_words at malloc_time_micros == 0
    mem_df = file_df[file_df['malloc_time_micros'] == 0]
    axs[0].bar(['FIP True', 'FIP False'], 
               [mem_df[mem_df['fip'] == True]['max_mem_words'].values[0],
                mem_df[mem_df['fip'] == False]['max_mem_words'].values[0]],
               color=['green', 'red'])
    axs[0].set_title('Max Memory')
    axs[0].set_ylabel('max_mem_words')
    # Better visualization
    axs[0].set_ylim(bottom=0)

    # Subplot 2: Line chart for exec_time_ms vs malloc_time_micros
    for fip_value, color in zip([True, False], ['green', 'red']):
        sub_df = file_df[file_df['fip'] == fip_value].sort_values(by='malloc_time_micros')
        axs[1].plot(sub_df['malloc_time_micros'], sub_df['exec_time_ms'], 
                    marker='o', label=f'FIP {fip_value}', color=color)
    
    axs[1].set_title('Execution Time vs Malloc Time')
    axs[1].set_xlabel('malloc_time_micros')
    axs[1].set_ylabel('exec_time_ms')
    # Better visualization
    axs[1].set_ylim(bottom=0)
    axs[1].legend()
    
    plt.tight_layout()
    plt.savefig(f"../{file_name}.png")
