node {
    def app

    stage('Clone repository') {
        checkout scm
    }

    stage('Build image') {
        app = docker.build("yogstation/feedback-normalization")
    }

    stage('Push image') {
        docker.withRegistry('https://registry.hub.docker.com', 'yogstation-docker-hub-credentials') {
            app.push("${env.BUILD_NUMBER}")
            app.push("latest")
        }
    }

    stage('Deploy') {
        withKubeConfig([credentialsId: 'yogstation-kubeconf-credentials']) {
            sh 'kubectl set image --namespace yogbot deployment/feedback-normalization feedback-normalization=yogstation/feedback-normalization:${BUILD_NUMBER}'
        }   
    }
}