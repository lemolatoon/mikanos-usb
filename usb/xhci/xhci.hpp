/**
 * @file usb/xhci/xhci.hpp
 *
 * xHCI ホストコントローラ制御用クラス．
 */

#pragma once

#include "error.hpp"
#include "usb/xhci/registers.hpp"
#include "usb/xhci/context.hpp"
#include "usb/xhci/ring.hpp"
#include "usb/xhci/port.hpp"
#include "usb/xhci/devmgr.hpp"

namespace usb::xhci
{
  std::unique_ptr<Controller> new_controller(uintptr_t mmio_base);
  class Controller
  {
  public:
    Controller(uintptr_t mmio_base);
    std::unique_ptr<Error> Initialize();
    std::unique_ptr<Error> Run();
    Ring *CommandRing() { return &cr_; }
    EventRing *PrimaryEventRing() { return &er_; }
    DoorbellRegister *DoorbellRegisterAt(uint8_t index);
    std::unique_ptr<Port> PortAt(uint8_t port_num)
    {
      return std::make_unique<Port>(port_num, PortRegisterSets()[port_num - 1]);
    }
    uint8_t MaxPorts() const { return max_ports_; }
    DeviceManager *DeviceManager() { return &devmgr_; }

  private:
    static const size_t kDeviceSize = 8;

    const uintptr_t mmio_base_;
    CapabilityRegisters *const cap_;
    OperationalRegisters *const op_;
    const uint8_t max_ports_;

    class DeviceManager devmgr_;
    Ring cr_;
    EventRing er_;

    InterrupterRegisterSetArray InterrupterRegisterSets() const
    {
      return {mmio_base_ + cap_->RTSOFF.Read().Offset() + 0x20u, 1024};
    }

    PortRegisterSetArray PortRegisterSets() const
    {
      return {reinterpret_cast<uintptr_t>(op_) + 0x400u, max_ports_};
    }

    DoorbellRegisterArray DoorbellRegisters() const
    {
      return {mmio_base_ + cap_->DBOFF.Read().Offset(), 256};
    }
  };

  std::unique_ptr<Error> ConfigurePort(Controller &xhc, Port &port);
  std::unique_ptr<Error> ConfigureEndpoints(Controller &xhc, Device &dev);

  /** @brief イベントリングに登録されたイベントを高々1つ処理する．
   *
   * xhc のプライマリイベントリングの先頭のイベントを処理する．
   * イベントが無ければ即座に Error::kSuccess を返す．
   *
   * @return イベントを正常に処理できたら Error::kSuccess
   */
  std::unique_ptr<Error> ProcessEvent(Controller &xhc);
}
