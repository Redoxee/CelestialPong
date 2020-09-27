using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class ComputeShaderHandler : MonoBehaviour
{
    [SerializeField]
    private ComputeShader computeShader = null;

    [SerializeField]
    private Texture2D initialStateTexture = null;

    int texSize = 256;
    RenderTexture[] rdTextures;
    int currentRdTexture = 0;
    Renderer myRenderer;

    private int mainKernel;

    const float RefreshRate = .0f;
    float timer = RefreshRate;

    private void Start()
    {
        this.rdTextures = new RenderTexture[2];

        for (int index = 0; index < this.rdTextures.Length; ++index)
        {
            this.rdTextures[index] = new RenderTexture(this.texSize, this.texSize, 24);
            this.rdTextures[index].enableRandomWrite = true;
            this.rdTextures[index].Create();
        }

        this.myRenderer = this.GetComponent<Renderer>();
        this.myRenderer.enabled = true;

        this.SetupComputeShader();
    }

    private void SetupComputeShader()
    {
        int initializationKernel = this.computeShader.FindKernel("CSInitialization");
        this.computeShader.SetTexture(initializationKernel,"InitialState" , this.initialStateTexture);
        this.computeShader.SetTexture(this.mainKernel, "PreviousState", this.rdTextures[this.currentRdTexture]);
        this.computeShader.SetTexture(initializationKernel, "Result", this.rdTextures[this.currentRdTexture]);
        this.computeShader.Dispatch(initializationKernel, this.texSize / 8, this.texSize / 8, 4);

        this.mainKernel = this.computeShader.FindKernel("CSMain");

    }

    private void Update()
    {

        this.timer -= Time.deltaTime;
        if (this.timer > 0)
        {
            return;
        }

        this.timer = RefreshRate;

        this.computeShader.SetTexture(this.mainKernel, "PreviousState", this.rdTextures[this.currentRdTexture]);
        this.currentRdTexture = (this.currentRdTexture + 1) % this.rdTextures.Length;
        this.computeShader.SetTexture(this.mainKernel, "Result", this.rdTextures[this.currentRdTexture]);
        this.computeShader.Dispatch(mainKernel, this.texSize / 8, this.texSize / 8, 1);
        this.myRenderer.material.SetTexture("_MainTex", this.rdTextures[this.currentRdTexture]);
    }
}
