# 🚀 How to Deploy grio on Hugging Face Spaces

Deploying your interactive showcase to Hugging Face Spaces takes less than 2 minutes:

### Step 1: Create a new Space on Hugging Face
1. Go to [https://huggingface.co/new-space](https://huggingface.co/new-space)
2. Choose a **Space name** (e.g. `grio-showcase` or `grio`)
3. Select **License**: `MIT`
4. Select **Space SDK**: **`Docker`** -> **`Blank`**
5. Visibility: **`Public`**
6. Click **Create Space**

---

### Step 2: Push the code to your Space

In your terminal:

```bash
# 1. Clone your newly created HF Space (replace <your-hf-username> and <space-name>)
git clone https://huggingface.co/spaces/<your-hf-username>/<space-name> hf-space-repo
cd hf-space-repo

# 2. Copy the contents of the deploy/hf-space/ folder and the project files
# - Copy Dockerfile and README.md from deploy/hf-space/
# - Copy Cargo.toml, Cargo.lock, and crates/ from the grio root

# 3. Commit and push
git add .
git commit -m "feat: deploy grio interactive showcase"
git push
```

Hugging Face will automatically trigger the build and your interactive demo will be live at:  
👉 `https://huggingface.co/spaces/<your-hf-username>/<space-name>`
