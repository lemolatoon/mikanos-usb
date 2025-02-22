/**
 * @file usb/device.hpp
 *
 * USB デバイスを表すクラスと関連機能．
 */

#pragma once

#include <array>

#include "error.hpp"
#include "usb/setupdata.hpp"
#include "usb/endpoint.hpp"
#include "usb/arraymap.hpp"

namespace usb
{
  class ClassDriver;

  class Device
  {
  public:
    virtual ~Device();
    virtual std::unique_ptr<Error> ControlIn(EndpointID ep_id, SetupData setup_data,
                                             void *buf, int len, ClassDriver *issuer);
    virtual std::unique_ptr<Error> ControlOut(EndpointID ep_id, SetupData setup_data,
                                              const void *buf, int len, ClassDriver *issuer);
    virtual std::unique_ptr<Error> InterruptIn(EndpointID ep_id, void *buf, int len);
    virtual std::unique_ptr<Error> InterruptOut(EndpointID ep_id, void *buf, int len);

    std::unique_ptr<Error> StartInitialize();
    bool IsInitialized() { return is_initialized_; }
    EndpointConfig *EndpointConfigs() { return ep_configs_.data(); }
    int NumEndpointConfigs() { return num_ep_configs_; }
    std::unique_ptr<Error> OnEndpointsConfigured();

    uint8_t *Buffer() { return buf_.data(); }

  protected:
    std::unique_ptr<Error> OnControlCompleted(EndpointID ep_id, SetupData setup_data,
                                              const void *buf, int len);
    std::unique_ptr<Error> OnInterruptCompleted(EndpointID ep_id, const void *buf, int len);

  private:
    /** @brief エンドポイントに割り当て済みのクラスドライバ．
     *
     * 添字はエンドポイント番号（0 - 15）．
     * 添字 0 はどのクラスドライバからも使われないため，常に未使用．
     */
    std::array<ClassDriver *, 16> class_drivers_{};

    std::array<uint8_t, 256> buf_{};

    // following fields are used during initialization
    uint8_t num_configurations_;
    uint8_t config_index_;

    std::unique_ptr<Error> OnDeviceDescriptorReceived(const uint8_t *buf, int len);
    std::unique_ptr<Error> OnConfigurationDescriptorReceived(const uint8_t *buf, int len);
    std::unique_ptr<Error> OnSetConfigurationCompleted(uint8_t config_value);

    bool is_initialized_ = false;
    int initialize_phase_ = 0;
    std::array<EndpointConfig, 16> ep_configs_;
    int num_ep_configs_;
    std::unique_ptr<Error> InitializePhase1(const uint8_t *buf, int len);
    std::unique_ptr<Error> InitializePhase2(const uint8_t *buf, int len);
    std::unique_ptr<Error> InitializePhase3(uint8_t config_value);
    std::unique_ptr<Error> InitializePhase4();

    /** OnControlCompleted の中で要求の発行元を特定するためのマップ構造．
     * ControlOut または ControlIn を発行したときに発行元が登録される．
     */
    ArrayMap<SetupData, ClassDriver *, 4> event_waiters_{};
  };

  std::unique_ptr<Error> GetDescriptor(Device &dev, EndpointID ep_id,
                                       uint8_t desc_type, uint8_t desc_index,
                                       void *buf, int len, bool debug = false);
  std::unique_ptr<Error> SetConfiguration(Device &dev, EndpointID ep_id,
                                          uint8_t config_value, bool debug = false);
}
