from graphs import *

# make_graph("./trl-experiments/LineRider3D-Env-v0/PPO/d1cd338e-6255-4feb-afe8-7cbec17f8ab4.tlrx")

# get_eval_line("./trl-experiments/LineRider3D-Env-v0/PPO", "./trl-experiments/LineRider3D-Env-v0/PPO/ce432a66-54b7-4c4a-b9d1-430d83fc64cf.tlrx")

# compare_plots([
#   ("./trl-experiments/LineRider3D-Env-v0/PPO/d1cd338e-6255-4feb-afe8-7cbec17f8ab4.tlrx","two"),
#   ("./trl-experiments/LineRider3D-Env-v0/PPO/65b2559e-0d75-40bf-b20e-ee00fb98e598.tlrx","One")
# ])

MANUAL_HEURISTIC = "[254]"

REWARD_HEURISTIC = "[24,18,1,0]"

REWARD_REGULAR = "[15,18,21,1,0,23]"
REWARD_REGULAR_BOOST = "[25,15,18,21,1,0,23]"
REWARD_AIR = f"[15,18,21,1,0,23,12]"

# add action space for _notp stuff
EXP_HEU_DOWN = ({"reward_type": MANUAL_HEURISTIC, "target_type": "1", "action_type": "6"}, "Heu-Down", False)
EXP_HEU_UP = ({"reward_type": MANUAL_HEURISTIC, "target_type": "4", "action_type": "6"}, "Heu-Up", False)
EXP_HEU_SAME = ({"reward_type": MANUAL_HEURISTIC, "target_type": "5", "action_type": "6"}, "Heu-Same", False)
EXP_MIMIC_DOWN = ({"reward_type": REWARD_HEURISTIC, "target_type": "1", "action_type": "6"}, "Mimic-Down", False)
EXP_DOWN = ({"reward_type": REWARD_REGULAR, "target_type": "1", "action_type": "6"}, "Down", False)
EXP_MIMIC_SAME = ({"reward_type": REWARD_HEURISTIC, "target_type": "5", "action_type": "6"}, "Mimic-Same", False)
EXP_SAME = ({"reward_type": REWARD_REGULAR_BOOST, "target_type": "5", "action_type": "6"}, "Same", False)
EXP_MIMIC_UP = ({"reward_type": REWARD_HEURISTIC, "target_type": "4", "action_type": "6"}, "Mimic-Up", False)
EXP_UP = ({"reward_type": REWARD_REGULAR_BOOST, "target_type": "4", "action_type": "6"}, "Up", False)
EXP_AIR = ({"reward_type": REWARD_AIR, "target_type": "4", "action_type": "6"}, "Regular-Up", False)

# put in seperate folder!
EXP_MIMIC_DOWN_20 = ({"reward_type": REWARD_HEURISTIC, "target_type": "1", "max_width": "20"}, "Mimic-Down-20", False)
EXP_MIMIC_DOWN_30 = ({"reward_type": REWARD_HEURISTIC, "target_type": "1", "max_width": "30"}, "Mimic-Down-30", False)
EXP_MIMIC_DOWN_40 = ({"reward_type": REWARD_HEURISTIC, "target_type": "1", "max_width": "40"}, "Mimic-Down-40", False)
EXP_MIMIC_DOWN_50 = ({"reward_type": REWARD_HEURISTIC, "target_type": "1", "max_width": "50"}, "Mimic-Down-50", False)

CHK_D10 = ({"reward_type": "[15,18,21,1,0,23,26]", "target_type": "7", "max_width": "10"}, "Chk-Down-10", False)
CHK_D20 = ({"reward_type": "[15,18,21,1,0,23,26]", "target_type": "7", "max_width": "20"}, "Chk-Down-20", False)
CHK_U10 = ({"reward_type": "[15,18,21,1,0,23,26]", "target_type": "8", "max_width": "10"}, "Chk-Up-10", False)
CHK_U20 = ({"reward_type": "[15,18,21,1,0,23,26]", "target_type": "8", "max_width": "20"}, "Chk-Up-20", False)
CHK_H10 = ({"reward_type": "[254]", "target_type": "7", "max_width": "10"}, "Baseline-Chk-Down-10", False)
CHK_H20 = ({"reward_type": "[254]", "target_type": "7", "max_width": "20"}, "Baseline-Chk-Down-20", False)
CHK_HU10 = ({"reward_type": "[254]", "target_type": "8", "max_width": "10"}, "Baseline-Chk-Up-10", False)
CHK_HU20 = ({"reward_type": "[254]", "target_type": "8", "max_width": "20"}, "Baseline-Chk-Up-20", False)

TP_S = ({"reward_type": "[15,18,21,1,0,23,5]", "target_type": "1", "max_width": "10"}, "Shortest", False)
TP_T = ({"reward_type": "[15,18,21,1,0,23,11]", "target_type": "1", "max_width": "10"}, "Touch", False)
TP_A = ({"reward_type": "[15,18,21,1,0,23,12]", "target_type": "1", "max_width": "10"}, "Air", False)
TP_SP = ({"reward_type": "[15,18,21,1,0,23,6,7,17]", "target_type": "1", "max_width": "10"}, "Speed", False)
TP_R = ({"reward_type": "[15,18,21,1,0,23,9]", "target_type": "1", "max_width": "10"}, "Rotation", False)
TP_TSP = ({"reward_type": "[15,18,21,1,0,23,11,6,7,17]", "target_type": "1", "max_width": "10"}, "Touch&Speed", False)
TP_ASP = ({"reward_type": "[15,18,21,1,0,23,12,6,7,17]", "target_type": "1", "max_width": "10"}, "Air&Speed", False)

SIZE10 = ({"reward_type": REWARD_HEURISTIC, "target_type": "1", "action_type": "6", "observation_type": "3", "max_width": "10"}, "Size-10", False)
SIZE20 = ({"reward_type": REWARD_HEURISTIC, "target_type": "1", "action_type": "6", "observation_type": "3", "max_width": "20"}, "Size-20", False)
SIZE30 = ({"reward_type": REWARD_HEURISTIC, "target_type": "1", "action_type": "6", "observation_type": "3", "max_width": "30"}, "Size-30", False)
SIZE40 = ({"reward_type": REWARD_HEURISTIC, "target_type": "1", "action_type": "6", "observation_type": "3", "max_width": "40"}, "Size-40", False)

SIZE10S = ({"reward_type": REWARD_REGULAR, "target_type": "1", "action_type": "6", "observation_type": "4", "max_width": "10"}, "Size-10-Sliding", False)
SIZE20S = ({"reward_type": REWARD_REGULAR, "target_type": "1", "action_type": "6", "observation_type": "4", "max_width": "20"}, "Size-20-Sliding", False)
SIZE30S = ({"reward_type": REWARD_REGULAR, "target_type": "1", "action_type": "6", "observation_type": "4", "max_width": "30"}, "Size-30-Sliding", False)
SIZE40S = ({"reward_type": REWARD_REGULAR, "target_type": "1", "action_type": "6", "observation_type": "4", "max_width": "40"}, "Size-40-Sliding", False)


TRANSFER_SIZE10S_20S = (SIZE20S[0], "TR-S10-S20", "sliding10")
TRANSFER_SIZE10S_20S_SAME = ({"reward_type": REWARD_REGULAR_BOOST, "target_type": "1", "action_type": "6", "observation_type": "4", "max_width": "20"}, "TR-S10-S20_same", "sliding10")
TRANSFER_SIZE10S_20S_CHK = (CHK_D20[0], "TR-S10-S20_ChkDown", "sliding10")


TRANSFER_SIZE10S_20S_L = (SIZE20S[0], "TR-S10-S20", "sliding10")
TRANSFER_SIZE20S_30S = (SIZE30S[0], "TR-S20-S30", "sliding20")


SIZE10H = ({"reward_type": MANUAL_HEURISTIC, "max_width": "10"}, "Size-10-Heu", False)
SIZE20H = ({"reward_type": MANUAL_HEURISTIC, "max_width": "20"}, "Size-20-Heu", False)
SIZE30H = ({"reward_type": MANUAL_HEURISTIC, "max_width": "30"}, "Size-30-Heu", False)
SIZE40H = ({"reward_type": MANUAL_HEURISTIC, "max_width": "40"}, "Size-40-Heu", False)


TRANSFER_SAME_TOPBOT = (EXP_DOWN[0], "TR-Same-Down", "same")
TRANSFER_SAME_CHK = (CHK_D10[0], "TR-Same-Chk-Down", "same")
TRANSFER_SAME_UP = ({"target_type": "4", "action_type": "6"}, "TR-Same-Up", "same")

TRANSFER_BOT_SAME = (EXP_SAME[0], "TR-Down-Same", "topbot")
TRANSFER_BOT_UP = ({"target_type": "4", "action_type": "6"}, "TR-Down-Up", "topbot")
TRANSFER_BOT_CHK = (CHK_D10[0], "TR-Down-Chk-Down", "topbot")

# dir_to_use = "./trl-experiments/LineRider3D-Env-v0/PPO"
# dir_to_use = "./trl-experiments/LineRider3D-Env-v0/size_and_basic"
# dir_to_use = "./trl-experiments/LineRider3D-Env-v0/chkpointntrack"
# dir_to_use = "../../0exp/basic"
configs_to_plot = [
  # ("../../0exp/basic", "fig_basic_training", [EXP_DOWN[1], EXP_MIMIC_UP[1], EXP_UP[1], EXP_MIMIC_SAME[1], EXP_SAME[1], EXP_MIMIC_DOWN[1]], [EXP_DOWN, EXP_MIMIC_UP, EXP_UP, EXP_MIMIC_SAME, EXP_SAME, EXP_MIMIC_DOWN]),
  # ("../../0exp/chk", "fig_chk", [CHK_D10[1],CHK_H10[1],CHK_U10[1],CHK_HU10[1],CHK_D20[1],CHK_H20[1],CHK_U20[1],CHK_HU20[1]], [CHK_HU20, CHK_U20, CHK_H20, CHK_D20, CHK_HU10, CHK_U10, CHK_H10, CHK_D10]),
  # ("../../0exp/tracktype", "fig_tt", [TP_T[1], TP_A[1], TP_SP[1], TP_TSP[1], TP_ASP[1]], [TP_T, TP_A, TP_SP, TP_TSP, TP_ASP]),
  # ("../../0exp/size", "fig_size", [SIZE10[1], SIZE10H[1], SIZE20[1], SIZE20H[1], SIZE30[1], SIZE30H[1], SIZE40[1], SIZE40H[1]], [SIZE10, SIZE20, SIZE30, SIZE40, SIZE10H, SIZE20H, SIZE30H, SIZE40H]),
  #("../../0exp/transfernbasic", "fig_transfer", [ EXP_DOWN[1], CHK_D10[1], EXP_SAME[1], TRANSFER_SAME_TOPBOT[1],TRANSFER_SAME_CHK[1], TRANSFER_BOT_CHK[1], TRANSFER_BOT_SAME[1], TRANSFER_SAME_UP[1], TRANSFER_BOT_UP[1]], [TRANSFER_SAME_TOPBOT, EXP_DOWN, TRANSFER_SAME_CHK, TRANSFER_BOT_CHK, CHK_D10, TRANSFER_BOT_SAME, EXP_SAME, TRANSFER_SAME_UP, TRANSFER_BOT_UP]),
  # ("../../0exp/transfernbasic", "fig_transfernochk", [EXP_DOWN[1], EXP_SAME[1], EXP_UP[1], TRANSFER_SAME_TOPBOT[1], TRANSFER_BOT_SAME[1], TRANSFER_SAME_UP[1], TRANSFER_BOT_UP[1]], [TRANSFER_SAME_TOPBOT, EXP_DOWN, TRANSFER_BOT_SAME, EXP_SAME, EXP_UP, TRANSFER_SAME_UP, TRANSFER_BOT_UP]),
  # ("../../0exp/transfernbasic", "fig_transfernochk", None, [TRANSFER_SAME_UP, TRANSFER_BOT_UP]),
  ("../../0exp/sliding", "fig_sliding", [], [SIZE10S, SIZE20S, SIZE30S, SIZE40S]),
  #("../../0exp/slidingtransfer1020short", "fig_transfer_sliding", [], [SIZE10S, SIZE20S, TRANSFER_SIZE10S_20S, TRANSFER_SIZE10S_20S_CHK, CHK_D20]),
  # ("../../0exp/slidingtransfercur", "fig_transfer_cur", [SIZE10S[1], SIZE20S[1], SIZE30S[1], TRANSFER_SIZE10S_20S_L[1], TRANSFER_SIZE20S_30S[1]], [SIZE10S, SIZE20S, SIZE30S, TRANSFER_SIZE10S_20S_L, TRANSFER_SIZE20S_30S]),
]

for (dir_to_use, graph_name, x_sort, config_to_plot) in configs_to_plot:
  # compare_configs(dir_to_use, config_to_plot, file_name=graph_name)
  compare_configs(dir_to_use, config_to_plot, plot_final_eval=True, file_name=graph_name, x_sort=x_sort)
  # compare_configs(dir_to_use, config_to_plot, plot_final_eval=True, file_name=graph_name, x_sort=x_sort, bar_err=True)
  # compare_configs_inbetwee_eval(dir_to_use, config_to_plot, file_name=f"{graph_name}_inbetween")
  # compare_behavior(dir_to_use, config_to_plot, plot_final_eval=True, file_name=graph_name)