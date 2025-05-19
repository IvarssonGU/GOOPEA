import matplotlib.pyplot as plt
import pandas as pd
import sys

df = pd.read_csv(sys.argv[1], skipinitialspace=True)
df.columns = df.columns.str.strip()

# Plotting
for file_name in df['file'].unique():
    file_df = df[df['file'] == file_name]

    fig, axs = plt.subplots(1, 2, figsize=(7, 5), gridspec_kw={'width_ratios': [1, 1.8]})
    fig.suptitle(f"{file_name}", fontsize=14)

    # Subplot 1: Bar chart for max_mem_words at malloc_time_micros == 0
    mem_df = file_df[file_df['malloc_time_micros'] == 0]
    axs[0].bar(['FIP', 'no FIP', 'Scoped RC'], 
               [mem_df[mem_df['fip'] == 'fip']['max_mem_words'].values[0],
                mem_df[mem_df['fip'] == 'nofip']['max_mem_words'].values[0],
                mem_df[mem_df['fip'] == 'sc_rc']['max_mem_words'].values[0]],
               color=['green', 'red', 'blue'],
               hatch=['', '\\\\', '..'])
    axs[0].set_title('Maximum Memory')
    axs[0].set_ylabel('maximum memory used (words)')
    # Better visualization
    axs[0].set_ylim(bottom=0)
    axs[0].set

    # Subplot 2: Line chart for exec_time_ms vs malloc_time_micros
    for fip_value in ['fip', 'nofip', 'sc_rc']:
        sub_df = file_df[file_df['fip'] == fip_value].sort_values(by='malloc_time_micros')
        if fip_value == 'fip':
            axs[1].plot(
                sub_df['malloc_time_micros'],
                sub_df['exec_time_ms'], 
                marker='o' if fip_value else 's',
                linestyle='-' if fip_value else '--',
                label='FIP',
                color='g'
            )
        elif fip_value == 'nofip':
            axs[1].plot(
                sub_df['malloc_time_micros'],
                sub_df['exec_time_ms'], 
                marker='s',
                linestyle='--',
                label='no FIP',
                color='r'
            )
        else:
            axs[1].plot(
                sub_df['malloc_time_micros'],
                sub_df['exec_time_ms'], 
                marker='v',
                linestyle=':',
                label='Scoped RC',
                color='b'
            )
            
    fipinstrs = mem_df[mem_df['fip'] == 'fip']['steps'].values[0]
    sc_rc_instrs = mem_df[mem_df['fip'] == 'sc_rc']['steps'].values[0]
    nonfipinstrs = mem_df[mem_df['fip'] == 'nofip']['steps'].values[0]
    plt.figtext(0.5, 0.01, f"FIP program running {fipinstrs} instructions. Non FIP running {nonfipinstrs}. Scoped RC running {sc_rc_instrs}.", ha="center", fontsize=9)
    
    axs[1].set_title('Execution Time vs Malloc Time')
    axs[1].set_xlabel('Artificial malloc delay (microseconds)')
    axs[1].set_ylabel('Execution time (milliseconds)')
    # Better visualization
    axs[1].set_ylim(bottom=0)
    axs[1].legend()
    
    plt.tight_layout()
    plt.savefig(f"../{file_name}.pdf")
